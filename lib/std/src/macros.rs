#[macro_export]
macro_rules! print {
    ($fmt:literal $($tt:tt)*) => {};
}

#[macro_export]
macro_rules! println {
    ($fmt:literal $($tt:tt)*) => {};
}
