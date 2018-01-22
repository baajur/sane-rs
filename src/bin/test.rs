extern crate byteorder;

use std::io::prelude::*;
use std::net::TcpStream;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

// 1.0.3
const SANE_VERSION: u32 = 0x01000300;

fn main() {
    let mut stream = TcpStream::connect("192.168.1.20:6566").expect("Failed to connect");

    let _ = stream.write_u32::<BigEndian>(0);
    let _ = stream.write(&[01, 00, 00, 03]);

    // zero-length array: username
    let _ = stream.write_u32::<BigEndian>(0);

    let status = stream.read_i32::<BigEndian>().unwrap();
    let version = stream.read_u32::<BigEndian>().unwrap();

    println!("Received status {}, version {:x}", status, version);
}
