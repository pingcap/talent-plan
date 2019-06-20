use std::{error, fmt, result};

use labcodec;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Encode(labcodec::EncodeError),
    Decode(labcodec::DecodeError),
    Rpc(labrpc::Error),
    NotLeader,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Error::Encode(ref e) => Some(e),
            Error::Decode(ref e) => Some(e),
            Error::Rpc(ref e) => Some(e),
            _ => None,
        }
    }
}

pub type Result<T> = result::Result<T, Error>;
