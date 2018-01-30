use std::convert::From;

#[derive(Debug, PartialEq)]
pub enum Status {
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

impl From<i32> for Status {
    fn from(val: i32) -> Status {
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
