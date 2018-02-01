#![feature(try_trait)]
#![feature(iterator_try_fold)]

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

/// Trait for types that can be read from a SANE network stream.
trait TryFromStream {
    fn try_from_stream(string: &mut TcpStream) -> Result<Self>
    where
        Self: std::marker::Sized;
}

struct Device {
    name: String,
    vendor: String,
    model: String,
    kind: String,
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

enum ValueType {
    Boolean,
    Integer,
    Fixed,
    String,
    Button,
    Group,
}

enum Unit {
    None,
    Pixel,
    Bit,
    MM,
    DPI,
    Percent,
    Microsecond,
}

enum Constraint {
    StringList(Vec<String>),
    IntegerList(Vec<i32>),
    Range { min: i32, max: i32, quant: i32 },
}

struct OptionDescriptor {
    name: String,
    title: String,
    desciption: String,
    kind: ValueType,
    unit: Unit,
    size: i32,
    cap: i32,
}

enum OpenResult {
    /// The device was successfully opened and a handle was returned
    Handle(i32),

    /// The device requires authentication, and an auth `resource`
    /// was returned.
    ///
    /// TODO: Test this case; I need someone with a device that would trigger this.
    AuthRequired(String),
}

fn init(stream: &mut TcpStream) {
    info!("Initializing connection");

    let _ = stream.write_u32::<BigEndian>(0);
    let _ = stream.write_u32::<BigEndian>(SANE_VERSION);

    // zero-length array: username
    //let _ = stream.write_u32::<BigEndian>(0);

    write_string("Foobar", stream).ok();

    // Make sure we received Success status
    check_success_status(stream).ok();

    let version = stream.read_u32::<BigEndian>().unwrap();

    println!("Connection initiated, version {:x}", version);
}

fn request_device_list(stream: &mut TcpStream) -> Result<Vec<Device>> {
    info!("Requesting device list");

    // Send Command
    stream.write_i32::<BigEndian>(1).ok();

    // Make sure we received Success status
    check_success_status(stream)?;

    // Read the array of devices
    read_array(stream, Device::try_from_stream)
}

fn open_device(device: &Device, stream: &mut TcpStream) -> Result<OpenResult> {
    info!("Opening device '{}'", device.name);

    // Send Command
    stream.write_i32::<BigEndian>(2).ok();

    // Send name of device to open
    write_string(&device.name, stream)?;

    // Make sure we received Success status
    check_success_status(stream)?;

    let handle = stream.read_i32::<BigEndian>().unwrap();
    let resource = read_string(stream)?;

    match resource {
        // If no resource is returned, the device was successfully opened
        None => Ok(OpenResult::Handle(handle)),
        // Otherwise, authentication is required
        Some(resource) => Ok(OpenResult::AuthRequired(resource)),
    }
}

fn close_device(handle: i32, stream: &mut TcpStream) {
    info!("Closing device using handle: {}", handle);

    // Send Command
    stream.write_i32::<BigEndian>(3).ok();

    // Send handle
    stream.write_i32::<BigEndian>(handle).ok();

    // Receive dummy
    let dummy = stream.read_i32::<BigEndian>().unwrap();
    debug!("Received dummy value {}", dummy);
}

/*
fn get_option_descriptors(handle: i32, stream: &mut TcpStream) {
    // Send Command
    stream.write_i32::<BigEndian>(4).ok();

    read_array(stream, builder)
}*/

fn read_string(stream: &mut TcpStream) -> Result<Option<String>> {
    let size = stream.read_i32::<BigEndian>().unwrap();

    if size <= 0 {
        return Ok(None);
    }

    String::from_utf8(
        stream
            // Read the number of bytes equal to the given size
            .take(u64::from(size as u32))
            .bytes()
            // Stop reading if we encounter an error or a null byte
            .take_while(|byte| byte.is_ok() && byte.as_ref().unwrap() != &0x00u8)
            // We're now guaranteed to not have an Err result, so unwrap to just a u8
            .map(|byte| byte.unwrap())
            // Collect into a Vec<u8>
            .collect(),
    ).map_err(|err| err.into())
        .map(|s| Some(s)) // Convert our Result<String> into Result<Option<String>>
}

fn write_string<S>(string: S, stream: &mut TcpStream) -> Result<()>
where
    S: AsRef<str>,
{
    use std::iter::repeat;
    // Get the &str
    let string = string.as_ref();

    // Make sure the length of the string fits into 32 bits
    // Worst case, usize is < 32 bits, in which case, the length definitely fits.
    if string.len() > i32::max_value() as usize {
        return Err(Error::BadNetworkDataError(format!(
            "String length of {} exceeds maximum possible length of {}!",
            string.len(),
            i32::max_value()
        )));
    }

    let length = string.len() as i32;

    // Double check that we didn't cut the string short
    assert!(string.len() == length as usize);

    let length = length + 1;

    stream.write_i32::<BigEndian>(length).ok();
    stream.write_all(string.as_bytes()).ok();
    stream.write(&vec![0x00u8]);

    Ok(())
}

fn read_array<F, T>(stream: &mut TcpStream, builder: F) -> Result<Vec<T>>
where
    F: Fn(&mut TcpStream) -> Result<T>,
{
    // Read pointer list:
    let size = stream.read_i32::<BigEndian>().unwrap();

    info!("Received array of size {}", size);

    (0..size)
        .map(|i| {
            let is_null = stream.read_i32::<BigEndian>().unwrap();

            // arrays are null terminated, but it's weird the null is included in the array's length
            assert!(
                i != size - 1 || is_null != 0,
                "Failed assumption of null terminator: {} = ({} - 1) and is_null is {}",
                i,
                size,
                is_null
            );

            debug!("Reading array element...");

            match is_null {
                0 => Ok(Some(builder(stream)?)),
                _ => Ok(None),
            }
        })
        .try_fold(Vec::new(), |mut arr, element: Result<Option<T>>| {
            debug!("Folding array element...");
            // Propagate an Err values up to the outer Result,
            // and filter out any None elements.
            if let Some(e) = element? {
                arr.push(e)
            }
            Ok(arr)
        })
}

fn read_status(stream: &mut TcpStream) -> Result<Status> {
    Ok(Status::from(stream.read_i32::<BigEndian>()?))
}

/// Read response status from `stream` and return Err if the status is
/// any value other than `Status::Success`.
fn check_success_status(stream: &mut TcpStream) -> Result<()> {
    match read_status(stream)? {
        Status::Success => Ok(()),
        err => Err(err.into()),
    }
}

fn main() {
    pretty_env_logger::init();

    let mut stream = TcpStream::connect("192.168.1.20:6566").expect("Failed to connect");
    stream.set_nodelay(true);

    init(&mut stream);

    let devices = request_device_list(&mut stream).unwrap();

    let device = devices
        .iter()
        .inspect(|device| {
            info!(
                "{} - {} - {} - {}",
                device.name, device.vendor, device.model, device.kind
            )
        })
        .take(1)
        .next()
        .unwrap();

    let handle = match open_device(&device, &mut stream) {
        Ok(result) => match result {
            OpenResult::Handle(handle) => {
                println!("Received handle {}", handle);
                Some(handle)
            }
            OpenResult::AuthRequired(resource) => {
                println!("Received authentication resource {}", resource);
                None
            }
        },
        Err(e) => {
            error!("{:?}", e);
            None
        }
    };

    println!("Closing device {}", &device.name);
    close_device(handle.unwrap(), &mut stream);
}
