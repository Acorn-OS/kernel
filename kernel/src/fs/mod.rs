pub mod impls;

use crate::util::locked::{Locked, ManualLock};
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

#[derive(Debug)]
pub enum Error {
    InvalidOperation(String),
    InvalidFile(String),
    NoSuchPath(String),
    InvalidMountPoint(String),
    InvalidPath(String),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&match self {
            Self::InvalidOperation(str) => format!("invalid operation: '{str}'"),
            Self::InvalidFile(str) => format!("invalid file: '{str}'"),
            Self::NoSuchPath(str) => format!("no such path: '{str}'"),
            Self::InvalidMountPoint(str) => format!("invalid mount point: {str}"),
            Self::InvalidPath(str) => format!("invalid path: '{str}'"),
        })
    }
}

impl core::error::Error for Error {}

pub type Result<T> = core::result::Result<T, Error>;

pub struct VNode {
    lock: ManualLock,
    write: fn(),
    read: fn(),
    close: fn(),
    ioctl: fn(),
    id: u64,
}

pub struct Metadata {
    pub name: String,
    pub size: usize,
}

pub trait Fs {
    fn open(&mut self, path: &str) -> Result<VNode>;

    fn ls(&mut self, path: &str) -> Result<Vec<Metadata>>;
}

struct MountPoint {
    name: String,
    fs: Box<dyn Fs>,
}

struct Vfs {
    root_mount: Option<Box<dyn Fs>>,
    mount_points: BTreeMap<String, MountPoint>,
}

impl Vfs {
    // standard functions.

    fn mount(&mut self, name: &str, fs: Box<dyn Fs>) -> Result<()> {
        if Self::is_mount_name_root(name) {
            self.root_mount = Some(fs)
        } else {
            let string = name.to_string();
            self.mount_points
                .insert(string.clone(), MountPoint { name: string, fs });
        }
        Ok(())
    }

    fn umount(&mut self, name: &str) -> Result<()> {
        if self.mount_points.contains_key(name) {
            Ok(())
        } else {
            Err(Error::InvalidMountPoint(name.to_string()))
        }
    }

    fn open(&mut self, path: &str) -> Result<VNode> {
        let (mount, path) = self.resolve_path(path)?;
        todo!()
    }

    fn ls(&mut self, path: &str) -> Result<Vec<Metadata>> {
        todo!()
    }

    // helper functions

    fn resolve_path<'a>(&'a self, path: &'a str) -> Result<(&str, &str)> {
        if path.chars().next() == Some(':') {
            let find = path.find('/').ok_or(Error::InvalidPath(path.to_string()))?;
            unsafe {
                Ok((
                    &*core::ptr::from_raw_parts::<str>(path.as_ptr().add(1) as *const _, find - 1),
                    &*core::ptr::from_raw_parts::<str>(
                        path.as_ptr().add(find) as *const _,
                        path.len() - find,
                    ),
                ))
            }
        } else {
            Ok(("", path))
        }
    }

    fn is_mount_name_root(name: &str) -> bool {
        name == ""
    }
}

static VFS: Locked<Vfs> = Locked::new(Vfs {
    root_mount: None,
    mount_points: BTreeMap::new(),
});

pub fn mount(name: &str, fs: Box<dyn Fs>) -> Result<()> {
    VFS.lock().mount(name, fs)
}

pub fn umount(name: &str) -> Result<()> {
    VFS.lock().umount(name)
}

pub fn open(path: &str) -> Result<VNode> {
    VFS.lock().open(path)
}

pub fn ls(path: &str) -> Result<Vec<Metadata>> {
    VFS.lock().ls(path)
}

pub fn close(inode: VNode) -> Result<()> {
    //let _lock = inode.lock.lock_empty();
    Ok(())
}

pub fn write(inode: VNode, buf: &[u8]) -> Result<usize> {
    //let _lock = inode.lock.lock_empty();
    Ok(todo!())
}

pub fn read(inode: VNode, buf: &mut [u8]) -> Result<usize> {
    //let _lock = inode.lock.lock_empty();
    Ok(todo!())
}

pub fn ioctl(inode: VNode, request: u64, buf: &mut [u8]) -> Result<()> {
    //let _lock = inode.lock.lock_empty();
    Ok(todo!())
}
