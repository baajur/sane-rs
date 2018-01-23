extern crate byteorder;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;
extern crate sane;

use sane::status::Status;
use sane::error::Error;

use std::io::prelude::*;
use std::net::TcpStream;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

type Result<T> = std::result::Result<T, Error>;

// 1.0.3
const SANE_VERSION: u32 = 0x01000003;

struct Device {
    name: String,
    vendor: String,
    model: String,
    kind: String,
}

impl Device {
    pub fn from_stream(stream: &mut TcpStream) -> Self {
        Self {
            name: read_string(stream),
            vendor: read_string(stream),
            model: read_string(stream),
            kind: read_string(stream),
        }
    }
}

fn init(stream: &mut TcpStream) {
    info!("Initializing connection");

    let _ = stream.write_u32::<BigEndian>(0);
    let _ = stream.write_u32::<BigEndian>(SANE_VERSION);

    // zero-length array: username
    let _ = stream.write_u32::<BigEndian>(0);

    let status = stream.read_i32::<BigEndian>().unwrap();

    // TODO: Check status

    let version = stream.read_u32::<BigEndian>().unwrap();

    println!("Received status {}, version {:x}", status, version);
}

fn request_device_list(stream: &mut TcpStream) -> Result<Vec<Option<Device>>> {
    info!("Requesting device list");

    // Send Command
    stream.write_u32::<BigEndian>(1u32).ok();

    let status = Status::from(stream.read_u32::<BigEndian>().unwrap());

    if status != Status::Success {
        return Err(status.into());
    }

    // Read pointer list:
    let size = stream.read_u32::<BigEndian>().unwrap();

    info!("Received array of size {}", size);

    Ok((0..size)
        .map(|_| {
            let is_null = stream.read_u32::<BigEndian>().unwrap();

            match is_null {
                0 => Some(Device::from_stream(stream)),
                _ => None,
            }
        })
        .collect())
}

fn read_string(stream: &mut TcpStream) -> String {
    let size = stream.read_u32::<BigEndian>().unwrap();

    let mut string = String::new();

    stream
        .take(u64::from(size))
        .read_to_string(&mut string)
        .ok();

    string
}

fn main() {
    pretty_env_logger::init();

    let mut stream = TcpStream::connect("192.168.1.20:6566").expect("Failed to connect");

    init(&mut stream);

    let devices = request_device_list(&mut stream).unwrap();

    for device in devices {
        match device {
            Some(device) => println!(
                "{} - {} - {} - {}",
                device.name, device.vendor, device.model, device.kind
            ),
            None => println!("NULL"),
        }
    }
}
