use core::fmt::Display;

use crate::mm::heap;

#[derive(Debug)]
pub enum Error {
    HeapAllocationError(heap::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::HeapAllocationError(err) => {
                f.write_fmt(format_args!("heap allocation error: {err}"))
            }
        }
    }
}

impl core::error::Error for Error {}

impl From<heap::Error> for Error {
    fn from(value: heap::Error) -> Self {
        Self::HeapAllocationError(value)
    }
}

pub type Result<T> = core::result::Result<T, Error>;
