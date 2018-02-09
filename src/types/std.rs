use std::io::prelude::*;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use TryFromStream;
use Result;

impl TryFromStream for u8 {
    fn try_from_stream<S: Read>(stream: &mut S) -> Result<Self> {
        stream.read_u8().map_err(|e| e.into())
    }
}

impl TryFromStream for i32 {
    fn try_from_stream<S: Read>(stream: &mut S) -> Result<Self> {
        stream.read_i32::<BigEndian>().map_err(|e| e.into())
    }
}

impl TryFromStream for u32 {
    fn try_from_stream<S: Read>(stream: &mut S) -> Result<Self> {
        stream.read_u32::<BigEndian>().map_err(|e| e.into())
    }
}

impl TryFromStream for Option<String> {
    fn try_from_stream<S: Read>(stream: &mut S) -> Result<Self> {
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

impl<T> TryFromStream for Option<T>
where
    T: TryFromStream,
{
    fn try_from_stream<S: Read>(stream: &mut S) -> Result<Self> {
        let is_null = stream.read_i32::<BigEndian>().unwrap();

        match is_null {
            0 => Ok(Some(T::try_from_stream(stream)?)),
            _ => Ok(None),
        }
    }
}

impl<T> TryFromStream for Vec<T>
where
    T: TryFromStream + ::std::fmt::Debug,
{
    fn try_from_stream<S: Read>(stream: &mut S) -> Result<Self> {
        // Read pointer list:
        let size = stream.read_i32::<BigEndian>().unwrap();

        info!("Received array of size {}", size);

        (0..size)
            .map(|i| T::try_from_stream(stream))
            .try_fold(Vec::new(), |mut arr, element| {
                // Propagate an Err values up to the outer Result,
                debug!("Folding element: {:?}", element);
                arr.push(element?);
                Ok(arr)
            })
            .map(|mut vec| {
                // Remove the trailing empty value
                debug!("Dropping trailing null value from vec: {:?}", vec.last());
                vec.truncate((size - 1) as usize);
                vec
            })
    }
}
