use std::convert::From;
use status::Status;

#[derive(Debug)]
pub enum Error {
    SanedError(Status),
    BadNetworkDataError(String),
    FromUtf8Error(::std::string::FromUtf8Error),
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
