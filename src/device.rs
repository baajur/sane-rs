use std::net::TcpStream;
use TryFromStream;
use Result;

#[derive(Debug)]
pub struct Device {
    pub name: String,
    pub vendor: String,
    pub model: String,
    pub kind: String,
}

impl TryFromStream for Device {
    fn try_from_stream(stream: &mut TcpStream) -> Result<Self> {
        Ok(Self {
            name: <Option<String>>::try_from_stream(stream)??,
            vendor: <Option<String>>::try_from_stream(stream)??,
            model: <Option<String>>::try_from_stream(stream)??,
            kind: <Option<String>>::try_from_stream(stream)??,
        })
    }
}
