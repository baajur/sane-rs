use std::io::prelude::*;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use {TryFromStream, WriteToStream};
use Result;

impl TryFromStream for bool {
    fn try_from_stream<S: Read>(stream: &mut S) -> Result<Self> {
        // http://www.sane-project.org/html/doc011.html#s4.2.2
        Ok(stream.read_u32::<BigEndian>()? == 1)
    }
}

impl TryFromStream for u8 {
    fn try_from_stream<S: Read>(stream: &mut S) -> Result<Self> {
        stream.read_u8().map_err(|e| e.into())
    }
}

impl WriteToStream for u8 {
    fn write_to<S: Write>(&self, stream: &mut S) -> Result<()> {
        Ok(stream.write_u8(*self)?)
    }
}

impl TryFromStream for i32 {
    fn try_from_stream<S: Read>(stream: &mut S) -> Result<Self> {
        stream.read_i32::<BigEndian>().map_err(|e| e.into())
    }
}

impl WriteToStream for i32 {
    fn write_to<S: Write>(&self, stream: &mut S) -> Result<()> {
        Ok(stream.write_i32::<BigEndian>(*self)?)
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

impl<T> WriteToStream for Option<T>
where
    T: WriteToStream,
{
    fn write_to<S: Write>(&self, stream: &mut S) -> Result<()> {
        if self.is_none() {
            // Welcome to the weird choices of SANE.
            // Here. we'll learn about null pointers.
            //
            // * From section 5.1.1: "...a NULL pointer is encoded as a zero-length array."
            // * From section 5.1.2: "A pointer is encoded by a word that indicates whether
            //   the pointer is a NULL-pointer which is then followed by the value that the
            //   pointer points to (in the case of a non-NULL pointer; in the case of
            //   a NULL pointer, no bytes are encoded for the pointer value)."
            //
            // It took me _way_ too long to finally understand that instead of being _sane_
            // and just sending a 0x00000000 word, all values are preceeded by their size,
            // so to send a NULL, we must send a word of value 1 (0x00000001) followed
            // by a 0x00000000 word to indicate the pointer is null.

            //stream.write(&[00, 00, 00, 01, 00, 00, 00, 00])?;
            stream.write_i32::<BigEndian>(1)?;
            stream.write_i32::<BigEndian>(0)?;
            return Ok(());
        }

        Ok(stream.write_i32::<BigEndian>(0)?)
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

#[cfg(test)]
mod tests {
    use super::*;
    use mockstream::MockStream;

    #[test]
    fn test_read_option_string_vec() {
        let mut stream = MockStream::new();
        stream.push_bytes_to_read(&hex!(
            "0000000400000006436f6c6f7200000000054772617900000000084c696e656172740000000000"
        ));

        let result = <Vec<Option<String>>>::try_from_stream(&mut stream);

        assert!(result.is_ok());
        assert_eq!(
            vec![
                Some("Color".into()),
                Some("Gray".into()),
                Some("Lineart".into()),
            ],
            result.unwrap()
        );
    }

    #[test]
    fn test_read_int_vec() {
        let mut stream = MockStream::new();
        stream.push_bytes_to_read(&hex!("00000005000000040000004b000000960000012c00000258"));

        let result = <Vec<i32>>::try_from_stream(&mut stream);

        assert!(result.is_ok());
        assert_eq!(vec![4, 75, 150, 300, 600], result.unwrap());
    }

    #[test]
    fn send_a_none_option() {
        let mut stream = MockStream::new();
        let option: Option<i32> = None;

        option.write_to(&mut stream).unwrap();

        assert_eq!(
            &hex!("00000001 00000000"),
            stream.pop_bytes_written().as_slice()
        );
    }
}
