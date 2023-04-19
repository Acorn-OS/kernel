use core::ops::Deref;

pub struct OsStr {}

pub struct OsString {}

#[repr(transparent)]
pub struct Path(OsStr);

impl Path {}

#[repr(transparent)]
pub struct PathBuf(OsString);

impl Deref for PathBuf {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        todo!()
    }
}
