use std::{error, fmt, result};

use futures::channel::oneshot::Canceled;

use labcodec::{DecodeError, EncodeError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Unimplemented(String),
    Encode(EncodeError),
    Decode(DecodeError),
    Recv(Canceled),
    Timeout,
    Stopped,
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Error::Encode(ref e) => Some(e),
            Error::Decode(ref e) => Some(e),
            Error::Recv(ref e) => Some(e),
            _ => None,
        }
    }
}

pub type Result<T> = result::Result<T, Error>;
