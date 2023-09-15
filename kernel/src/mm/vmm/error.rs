use allocators::freelist;
use core::fmt::{self, Display};

#[derive(Debug)]
pub enum Error {
    AllocatorError(freelist::Error),
}

impl From<freelist::Error> for Error {
    fn from(value: freelist::Error) -> Self {
        Self::AllocatorError(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::AllocatorError(e) => f.write_fmt(format_args!("allocation error: {e}")),
        }
    }
}

impl core::error::Error for Error {}

pub type Result<T> = core::result::Result<T, Error>;
