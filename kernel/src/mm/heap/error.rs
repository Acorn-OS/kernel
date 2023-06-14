use core::fmt::Display;

#[derive(Debug)]
pub enum Error {
    OutOfSpace,
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::OutOfSpace => f.write_str("out of space"),
        }
    }
}

impl core::error::Error for Error {}

pub type Result<T> = core::result::Result<T, Error>;
