use serial;
use std::path::Path;

pub enum Address {
    Unicast(u8),
    Broadcast,
}

impl Address {
    pub fn to_byte(&self) -> u8 {
        match self {
            Unicast(n) => n,
            Broadcast  => 254,
        }
    }
}

pub struct Device {
    addr: Address,
    port: serial::unix::TTYPort,
}

impl Device {
    pub fn create(addr: Address, path: &Path) -> serial::Result<Self> {
        use serial::core::SerialPort;
        let mut port = serial::unix::TTYPort::open(path)?;
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

    pub fn send(&mut self, message: &str) -> std::Result<String, u8> {
        unimplemented!()
    }

    // pub fn query(&mut self, address: Address, )
}

fn main() -> std::io::Result<()> {
    let mut dev = Device::create(std::path::Path::new("/dev/ttyUSB0"))?;
    Ok(())
}
