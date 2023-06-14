pub struct RingBuf<T, const SIZE: usize> {
    buf: [T; SIZE],
    reader: usize,
    writer: usize,
}
