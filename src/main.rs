use serial;
use std::path::Path;
use std::io::Read;
use std::io::Write;

#[derive(Debug)]
pub enum Address {
    Unicast(u8),
    Broadcast,
}

impl Address {
    pub fn to_byte(&self) -> u8 {
        match self {
            Address::Unicast(n) => n.clone(),
            Address::Broadcast  => 254,
        }
    }

    pub fn to_string(&self) -> String {
        format!("{:03}", self.to_byte())
    }
}

pub struct Device {
    addr: Address,
    port: Box<dyn serial::core::SerialPort>,
}

#[derive(Debug)]
pub enum NAK {
    ZeroAdjustmentAtTooHighPressure,
    AtmoAdjustmentAtTooLowPressure,
    UnrecognizedMessage,
    InvalidArgument,
    ValueOutOfRange,
    CommandOrQueryCharacterInvalid,
    NotInSetupMode,
}

#[derive(Debug)]
pub enum Error {
    ParseError,
    IOError(std::io::Error),
    SerialError(serial::Error),
    Utf8Error(std::string::FromUtf8Error),
    NAK(NAK),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IOError(err)
    }
}

impl From<serial::Error> for Error {
    fn from(err: serial::Error) -> Error {
        Error::SerialError(err)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(err: std::string::FromUtf8Error) -> Error {
        Error::Utf8Error(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

fn parse_response(resp: &[u8]) -> Result<String> {
    let l = resp.len();
    if l < 10                    { Err(Error::ParseError)?; }
    if resp[0] != b'@'           { Err(Error::ParseError)?; }
    if !resp[1].is_ascii_digit() { Err(Error::ParseError)?; }
    if !resp[2].is_ascii_digit() { Err(Error::ParseError)?; }
    if !resp[3].is_ascii_digit() { Err(Error::ParseError)?; }
    if resp[l - 3] != b';'       { Err(Error::ParseError)?; }
    if resp[l - 2] != b'F'       { Err(Error::ParseError)?; }
    if resp[l - 1] != b'F'       { Err(Error::ParseError)?; }
    match (resp[4], resp[5], resp[6]) {
        (b'A', b'C', b'K') => {
            Ok(String::from_utf8(resp[7 .. l - 3].to_vec())?)
        },
        (b'N', b'A', b'K') => {
            Err(match &resp[7 .. l - 3] {
                b"8"   => Error::NAK(NAK::ZeroAdjustmentAtTooHighPressure),
                b"9"   => Error::NAK(NAK::AtmoAdjustmentAtTooLowPressure),
                b"160" => Error::NAK(NAK::UnrecognizedMessage),
                b"169" => Error::NAK(NAK::InvalidArgument),
                b"172" => Error::NAK(NAK::ValueOutOfRange),
                b"175" => Error::NAK(NAK::CommandOrQueryCharacterInvalid),
                b"180" => Error::NAK(NAK::NotInSetupMode),
                _      => Error::ParseError,
            })
        },
        _ => Err(Error::ParseError),
    }
}

impl Device {
    pub fn new(addr: Address, path: &Path) -> serial::Result<Self> {
        use serial::core::SerialPort;
        let mut port = serial::open(path)?;
        let settings = serial::PortSettings {
            baud_rate:    serial::BaudRate::Baud9600,
            char_size:    serial::CharSize::Bits8,
            parity:       serial::Parity::ParityNone,
            stop_bits:    serial::StopBits::Stop1,
            flow_control: serial::FlowControl::FlowNone,
        };
        port.configure(&settings)?;
        Ok(Device { addr: addr, port: Box::new(port) })
    }

    pub fn send(&mut self, message: &str) -> Result<String> {
        write!(
            self.port,
            "@{}{};FF",
            self.addr.to_string(),
            message,
        ).unwrap();
        let mut buffer = [0; 1000];
        let reference = std::io::Read::by_ref(&mut self.port);
        let n = reference.take(1000).read(&mut buffer[..]).unwrap();
        parse_response(&buffer[0 .. n])
    }

    pub fn query(&mut self, query: &str) -> Result<String> {
        self.send(&format!("{}?", query))
    }

    pub fn command(
        &mut self,
        command: &str,
        parameter: &str,
    ) -> Result<String> {
        self.send(&format!("{}!{}", command, parameter))
    }
}

fn main() -> std::io::Result<()> {
    let mut dev = Device::new(
        Address::Broadcast,
        std::path::Path::new("/dev/ttyUSB0"),
    )?;
    let mut i: i32 = 0;
    loop {
        println!("DEBUG {:04}: {}", i, dev.query("PR3").unwrap());
        i = i + 1;
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    //Ok(())
}
