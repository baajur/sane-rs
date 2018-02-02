use std::net::TcpStream;
use TryFromStream;
use Result;

pub struct Device {
    pub name: String,
    pub vendor: String,
    pub model: String,
    pub kind: String,
}

impl TryFromStream for Device {
    fn try_from_stream(stream: &mut TcpStream) -> Result<Self> {
        Ok(Self {
            name: String::try_from_stream(stream)?,
            vendor: String::try_from_stream(stream)?,
            model: String::try_from_stream(stream)?,
            kind: String::try_from_stream(stream)?,
        })
    }
}
