use super::{Error, INode, Metadata, Vfs};
use alloc::slice;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::mem::size_of;

const HEADER_NAME_LEN: usize = 32;

#[repr(C, packed)]
struct Header {
    entries: u64,
    /// length in bytes for the whole initrd.
    length: u64,
}

#[repr(C, packed)]
struct EntryHeader {
    start: u64,
    /// The entry's size in bytes, excluding the header.
    length: u64,
    name: [u8; HEADER_NAME_LEN],
}

#[repr(C, packed)]
pub struct InitrdFs {
    inner: *const u8,
}

impl InitrdFs {
    pub unsafe fn from_raw(initd_ptr: *const u8, length: usize) -> Self {
        assert!(length >= size_of::<Header>());
        let ret = Self { inner: initd_ptr };
        assert!(ret.get_header().length <= length as u64);
        ret
    }

    unsafe fn get_header(&self) -> &Header {
        &*(self.inner as *const Header)
    }

    unsafe fn get_entry_header(&self, entry: usize) -> &EntryHeader {
        &*(self.inner.add(size_of::<Header>()) as *const EntryHeader).add(entry)
    }

    unsafe fn fs_open(&self, name: &str) -> Result<u64, Error> {
        let header = self.get_header();
        info!("name: {name}");
        if name.len() > HEADER_NAME_LEN {
            return Err(Error::InvalidFile(
                "file name was too large for initrd fs".to_string(),
            ));
        }
        let mut name_array = [0; 32];
        for (i, b) in name.bytes().enumerate() {
            name_array[i] = b;
        }
        for i in 0..header.entries as usize {
            let entry = self.get_entry_header(i);
            if entry.name == name_array {
                return Ok(i as u64);
            }
        }
        return Err(Error::NoSuchPath(name.to_string()));
    }

    unsafe fn fs_read(&self, index: u64) -> Result<&[u8], Error> {
        let header = self.get_entry_header(index as usize);
        Ok(slice::from_raw_parts(
            (self.inner as u64 + header.start) as *const _,
            header.length as usize,
        ))
    }

    unsafe fn fs_ls(&self) -> Result<Vec<Metadata>, Error> {
        let count = self.get_header().entries;
        let mut vec = vec![];
        for i in 0..count as usize {
            let header = self.get_entry_header(i);
            let name = header
                .name
                .iter()
                .cloned()
                .take_while(|&c| c != 0)
                .map(|c| c as char)
                .collect();
            vec.push(Metadata {
                name,
                size: header.length as usize,
            });
        }
        Ok(vec)
    }
}

impl Vfs for InitrdFs {
    fn open(&mut self, path: &str) -> Result<INode, super::Error> {
        Ok(INode(unsafe { self.fs_open(path)? }))
    }

    fn write(&mut self, _: INode, _: &[u8]) -> Result<(), super::Error> {
        Err(Error::InvalidOperation(
            "initrd fs does not support writing operations".to_string(),
        ))
    }

    fn read(&mut self, node: INode) -> Result<alloc::vec::Vec<u8>, super::Error> {
        Ok(unsafe { self.fs_read(node.0)?.to_vec() })
    }

    fn close(&mut self, _: INode) -> Result<(), super::Error> {
        Ok(())
    }

    fn ls(&mut self, _: &str) -> Result<Vec<Metadata>, Error> {
        unsafe { self.fs_ls() }
    }
}
