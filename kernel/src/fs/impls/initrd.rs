use crate::fs::{Error, Fs, Metadata, Result, VNode};
use alloc::slice;
use alloc::string::ToString;
use alloc::vec::Vec;
use allocators::freelist::Node;
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

    unsafe fn fs_open(&self, name: &str) -> Result<Node> {
        let header = self.get_header();
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
                return todo!();
                //return Ok(i as u64);
            }
        }
        return Err(Error::NoSuchPath(name.to_string()));
    }

    unsafe fn fs_read(&self, index: u64) -> Result<&[u8]> {
        let header = self.get_entry_header(index as usize);
        Ok(slice::from_raw_parts(
            (self.inner as u64 + header.start) as *const _,
            header.length as usize,
        ))
    }

    unsafe fn fs_ls(&self) -> Result<Vec<Metadata>> {
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

impl Fs for InitrdFs {
    fn open(&mut self, path: &str) -> Result<VNode> {
        todo!();
        //Ok(unsafe { self.fs_open(path)? })
    }

    fn ls(&mut self, _: &str) -> Result<Vec<Metadata>> {
        unsafe { self.fs_ls() }
    }
}
