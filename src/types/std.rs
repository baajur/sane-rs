use std::io::prelude::*;
use std::net::TcpStream;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use TryFromStream;
use Result;

impl TryFromStream for i32 {
    fn try_from_stream(stream: &mut TcpStream) -> Result<Self> {
        stream.read_i32::<BigEndian>().map_err(|e| e.into())
    }
}

impl TryFromStream for Option<String> {
    fn try_from_stream(stream: &mut TcpStream) -> Result<Self> {
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
}

/// Shorthand for unwrap()-ing an Option<String> and just returning Err if None.
impl TryFromStream for String {
    fn try_from_stream(stream: &mut TcpStream) -> Result<Self> {
        Ok(<Option<String>>::try_from_stream(stream)??)
    }
}

impl<T> TryFromStream for Vec<T>
where
    T: TryFromStream,
{
    fn try_from_stream(stream: &mut TcpStream) -> Result<Self> {
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
                    0 => Ok(Some(T::try_from_stream(stream)?)),
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
}
