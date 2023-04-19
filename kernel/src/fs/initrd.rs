use super::{FD, VFS};

pub struct InitrdFS {
    fd_counter: FD,
}

impl VFS for InitrdFS {
    fn open(&mut self, path: &str) -> Result<super::FD, super::Error> {
        let fd = self.fd_counter;
        self.fd_counter += 1;
        Ok(fd)
    }

    fn write(&mut self, fd: super::FD, data: impl AsRef<[u8]>) -> Result<(), super::Error> {
        todo!()
    }

    fn read(&mut self, fd: super::FD) -> Result<alloc::vec::Vec<u8>, super::Error> {
        todo!()
    }

    fn close(&mut self, fd: super::FD) -> Result<(), super::Error> {
        todo!()
    }
}
