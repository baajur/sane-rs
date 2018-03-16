#![feature(try_trait)]
#![feature(iterator_try_fold)]
#[macro_use]
extern crate bitflags;
extern crate byteorder;
#[cfg(test)]
#[macro_use]
extern crate hex_literal;
#[macro_use]
extern crate log;
#[cfg(test)]
extern crate mockstream;

pub mod error;
pub mod status;
pub mod types;
mod device;

use std::io::prelude::*;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

pub use device::Device;
use error::Error;
use status::Status;
use types::*;

pub type Result<T> = std::result::Result<T, error::Error>;

// 1.0.3
const SANE_VERSION: u32 = 0x01000003;

/// Trait for types that can be read from a SANE network stream.
trait TryFromStream {
    fn try_from_stream<S: Read>(string: &mut S) -> Result<Self>
    where
        Self: std::marker::Sized;
}

pub trait WriteToStream {
    fn write_to<S: Write>(&self, stream: &mut S) -> Result<()>;
}

pub enum OpenResult {
    /// The device was successfully opened and a handle was returned
    Handle(i32),

    /// The device requires authentication, and an auth `resource`
    /// was returned.
    ///
    /// TODO: Test this case; I need someone with a device that would trigger this.
    AuthRequired(String),
}

#[derive(Debug)]
pub enum ControlAction {
    Get,
    Set,
    SetAutomatic,
}

impl AsRef<i32> for ControlAction {
    fn as_ref(&self) -> &i32 {
        match self {
            &ControlAction::Get => &0,
            &ControlAction::Set => &1,
            &ControlAction::SetAutomatic => &2,
        }
    }
}

pub fn init<S: Read + Write>(stream: &mut S) {
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

pub fn request_device_list<S: Read + Write>(stream: &mut S) -> Result<Vec<Device>> {
    info!("Requesting device list");

    // Send Command
    stream.write_i32::<BigEndian>(1).ok();

    // Make sure we received Success status
    check_success_status(stream)?;

    // Read the array of devices
    <Vec<Option<Device>>>::try_from_stream(stream).map(|dev_list| {
        dev_list.into_iter()
            // Filter out any None elements
            .filter(|d| d.is_some())
            // None elements are gone, so unwrap all values
            .map(|d| d.unwrap()).collect()
    })
}

pub fn open_device<S: Read + Write>(device: &Device, stream: &mut S) -> Result<OpenResult> {
    info!("Opening device '{}'", device.name);

    // Send Command
    stream.write_i32::<BigEndian>(2).ok();

    // Send name of device to open
    write_string(&device.name, stream)?;

    // Make sure we received Success status
    check_success_status(stream)?;

    let handle = stream.read_i32::<BigEndian>().unwrap();
    let resource = <Option<String>>::try_from_stream(stream)?;

    match resource {
        // If no resource is returned, the device was successfully opened
        None => Ok(OpenResult::Handle(handle)),
        // Otherwise, authentication is required
        Some(resource) => Ok(OpenResult::AuthRequired(resource)),
    }
}

pub fn close_device<S: Read + Write>(handle: i32, stream: &mut S) {
    info!("Closing device using handle: {}", handle);

    // Send Command
    stream.write_i32::<BigEndian>(3).ok();

    // Send handle
    stream.write_i32::<BigEndian>(handle).ok();

    // Receive dummy
    let dummy = stream.read_i32::<BigEndian>().unwrap();
    debug!("Received dummy value {}", dummy);
}

pub fn get_option_descriptors<S: Read + Write>(
    handle: i32,
    stream: &mut S,
) -> Result<Vec<Option<OptionDescriptor>>> {
    info!("Requesting options for device: {}", handle);

    // Send Command
    stream.write_i32::<BigEndian>(4).ok();

    // Send handle
    stream.write_i32::<BigEndian>(handle).ok();

    <_>::try_from_stream(stream)
}

pub fn control_option<S: Read + Write, V: WriteToStream>(
    stream: &mut S,
    handle: i32,
    option: u32,
    action: ControlAction,
    kind: &OptionDescriptor,
    value: Option<V>,
) -> Result<ControlOptionResult> {
    info!("Sending option control request of type {:?}", action);

    // Send Command
    stream.write_i32::<BigEndian>(5)?;

    stream.write_i32::<BigEndian>(handle)?;
    stream.write_u32::<BigEndian>(option)?;
    stream.write_i32::<BigEndian>(*action.as_ref())?;
    stream.write_i32::<BigEndian>(kind.into())?;
    stream.write_i32::<BigEndian>(kind.size())?;
    value.write_to(stream)?;

    // Await your reply

    println!("checking status");
    check_success_status(stream)?;

    let result = kind.read_value(stream)?;

    info!("Result: {:?}", result);

    let resource = <Option<String>>::try_from_stream(stream)?;

    // TODO Handle the case where a resource is returned
    assert!(resource.is_none()); // a hacky reminder.

    info!("\t| Res:   {:?}", resource);

    Ok(result)
}

fn write_string<S, I: Read + Write>(string: S, stream: &mut I) -> Result<()>
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

fn read_status<S: Read>(stream: &mut S) -> Result<Status> {
    Ok(Status::from(stream.read_i32::<BigEndian>()?))
}

/// Read response status from `stream` and return Err if the status is
/// any value other than `Status::Success`.
fn check_success_status<S: Read + Write>(stream: &mut S) -> Result<()> {
    match read_status(stream)? {
        Status::Success => Ok(()),
        err => Err(err.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockstream::MockStream;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_get_option() {
        // Don't really care about the actual contents here, for now,
        // since we're just testing how the function reads the stream.
        let kind = OptionDescriptor::Integer {
            name: "test".into(),
            title: "test".into(),
            description: "test".into(),
            unit: OptionUnit::None,
            size: 4,
            capabilities: Capabilities::default(),
            constraint: None,
        };

        let mut stream = MockStream::new();
        stream.push_bytes_to_read(&hex!(
            "00000000 00000000 00000001 00000004 00000001 00000019 00000000"
        ));

        let result = control_option::<_, u8>(&mut stream, 0, 0, ControlAction::Get, &kind, None);

        let expected = ControlOptionResult {
            value: Some(OptionValue::Integer(25)),
            info: ControlOptionSetInfo::default(),
        };

        assert!(result.is_ok());
        assert_eq!(expected, result.unwrap());
        assert_eq!(
            &hex!("00000005 00000000 00000000 00000000 00000001 00000004 00000001 00000000"),
            stream.pop_bytes_written().as_slice()
        );
    }
}
