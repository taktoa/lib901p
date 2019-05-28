use serial;
use std::path::Path;
use std::io::Read;
use std::io::Write;

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
    port: serial::SystemPort,
}

impl Device {
    pub fn create(addr: Address, path: &Path) -> serial::Result<Self> {
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
        Ok(Device { addr: addr, port: port })
    }

    pub fn send(&mut self, message: &str) -> Result<String, u8> {
        write!(
            self.port,
            "@{}{};FF",
            self.addr.to_string(),
            message,
        ).unwrap();
        let mut buffer = [0; 1000];
        let reference = std::io::Read::by_ref(&mut self.port);
        let n = reference.take(1000).read(&mut buffer[..]).unwrap();
        Ok(String::from_utf8(buffer.to_vec()).unwrap())
    }

    // pub fn query(&mut self, address: Address, )
}

fn main() -> std::io::Result<()> {
    let mut dev = Device::create(
        Address::Broadcast,
        std::path::Path::new("/dev/ttyUSB0"),
    )?;
    let mut i: i32 = 0;
    loop {
        println!("DEBUG {:04}: {}", i, dev.send("PR3?").unwrap());
        i = i + 1;
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    //Ok(())
}
