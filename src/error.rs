use std::convert::From;
use status::Status;

#[derive(Debug)]
pub enum Error {
    SanedError(Status),
    /// Error for WORD fields that are constrained to a fixed set of possible values,
    /// such as "type" fields with a value corresponding to a specific type.
    InvalidSaneFieldValue(String, i32),
    BadNetworkDataError(String),
    FromUtf8Error(::std::string::FromUtf8Error),
    IOError(::std::io::Error),
    NoneError(::std::option::NoneError),
}

impl From<Status> for Error {
    fn from(status: Status) -> Error {
        Error::SanedError(status)
    }
}

impl From<::std::string::FromUtf8Error> for Error {
    fn from(error: ::std::string::FromUtf8Error) -> Error {
        Error::FromUtf8Error(error)
    }
}

impl From<::std::io::Error> for Error {
    fn from(error: ::std::io::Error) -> Error {
        Error::IOError(error)
    }
}

impl From<::std::option::NoneError> for Error {
    fn from(error: ::std::option::NoneError) -> Error {
        Error::NoneError(error)
    }
}
