extern crate byteorder;

use std::convert::From;
use std::io::prelude::*;
use std::net::TcpStream;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

// 1.0.3
const SANE_VERSION: u32 = 0x01000003;

#[derive(Debug, PartialEq)]
enum Status {
    Success,
    Unsupported,
    Canceled,
    DeviceBusy,
    Invalid,
    EndOfFile,
    Jammed,
    NoDocuments,
    CoverOpen,
    IOError,
    OutOfMemory,
    AccessDenied,
}

impl From<u32> for Status {
    fn from(val: u32) -> Status {
        match val {
            00 => Status::Success,
            01 => Status::Unsupported,
            02 => Status::Canceled,
            03 => Status::DeviceBusy,
            04 => Status::Invalid,
            05 => Status::EndOfFile,
            06 => Status::Jammed,
            07 => Status::NoDocuments,
            08 => Status::CoverOpen,
            09 => Status::IOError,
            10 => Status::OutOfMemory,
            11 => Status::AccessDenied,
            n => panic!("Unknown status {}!", n),
        }
    }
}

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
    let _ = stream.write_u32::<BigEndian>(0);
    let _ = stream.write_u32::<BigEndian>(SANE_VERSION);

    // zero-length array: username
    let _ = stream.write_u32::<BigEndian>(0);

    let status = stream.read_i32::<BigEndian>().unwrap();

    // TODO: Check status

    let version = stream.read_u32::<BigEndian>().unwrap();

    println!("Received status {}, version {:x}", status, version);
}

fn request_device_list(stream: &mut TcpStream) -> Vec<Device> {
    // Send Command
    stream.write_u32::<BigEndian>(1u32).ok();

    let status = Status::from(stream.read_u32::<BigEndian>().unwrap());

    if status != Status::Success {
        panic!("Received status {:?}", status);
    }

    // Read pointer list:
    let size = stream.read_u32::<BigEndian>().unwrap();

    let mut devices = Vec::new();

    for i in 0..size {
        let is_null = stream.read_u32::<BigEndian>().unwrap();

        if is_null == 0 {
            devices.push(Device::from_stream(stream));
        }
    }

    devices
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
    let mut stream = TcpStream::connect("192.168.1.20:6566").expect("Failed to connect");

    init(&mut stream);

    let devices = request_device_list(&mut stream);

    for device in devices {
        println!(
            "{} - {} - {} - {}",
            device.name, device.vendor, device.model, device.kind
        );
    }
}
