use std::net::TcpStream;
use TryFromStream;
use read_string;
use read_array;
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
            name: read_string(stream)??,
            vendor: read_string(stream)??,
            model: read_string(stream)??,
            kind: read_string(stream)??,
        })
    }
}
