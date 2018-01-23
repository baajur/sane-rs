use std::convert::From;
use status::Status;

#[derive(Debug)]
pub enum Error {
    SanedError(Status),
}

impl From<Status> for Error {
    fn from(status: Status) -> Error {
        Error::SanedError(status)
    }
}
