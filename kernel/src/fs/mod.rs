pub mod initrd;

use alloc::vec::Vec;

type FD = u16;

pub enum Error {}

pub trait VFS {
    fn open(&mut self, path: &str) -> Result<FD, Error>;

    fn write(&mut self, fd: FD, data: impl AsRef<[u8]>) -> Result<(), Error>;

    fn read(&mut self, fd: FD) -> Result<Vec<u8>, Error>;

    fn close(&mut self, fd: FD) -> Result<(), Error>;
}

pub type DynVFS = alloc::boxed::Box<dyn VFS>;
