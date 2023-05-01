pub mod initrd;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug)]
pub enum Error {
    InvalidOperation(String),
    InvalidFile(String),
    NoSuchPath(String),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&match self {
            Error::InvalidOperation(e) => format!("invalid operation: {e}"),
            Error::InvalidFile(e) => format!("invalid file: {e}"),
            Error::NoSuchPath(e) => format!("no such path: {e}"),
        })
    }
}

impl core::error::Error for Error {}

pub struct INode(u64);

pub struct Metadata {
    pub name: String,
    pub size: usize,
}

pub trait Vfs {
    fn open(&mut self, path: &str) -> Result<INode, Error>;

    fn write(&mut self, node: INode, data: &[u8]) -> Result<(), Error>;

    fn read(&mut self, node: INode) -> Result<Vec<u8>, Error>;

    fn close(&mut self, node: INode) -> Result<(), Error>;

    fn ls(&mut self, path: &str) -> Result<Vec<Metadata>, Error>;
}

pub type VfsContainer = Box<dyn Vfs>;

pub fn new(fs: impl Vfs + 'static) -> VfsContainer {
    Box::new(fs)
}

pub fn open(vfs: &mut dyn Vfs, path: &str) -> Result<INode, Error> {
    vfs.open(path)
}

pub fn write(vfs: &mut dyn Vfs, node: INode, data: impl AsRef<[u8]>) -> Result<(), Error> {
    vfs.write(node, data.as_ref())
}

pub fn read(vfs: &mut dyn Vfs, node: INode) -> Result<Vec<u8>, Error> {
    vfs.read(node)
}

pub fn close(vfs: &mut dyn Vfs, node: INode) -> Result<(), Error> {
    vfs.close(node)
}
