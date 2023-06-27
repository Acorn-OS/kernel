use core::mem::ManuallyDrop;

pub struct RingBuf<T, const SIZE: usize> {
    buf: [ManuallyDrop<T>; SIZE],
    reader: usize,
    writer: usize,
}

impl<T, const SIZE: usize> RingBuf<T, SIZE> {
    const _ASSERT: () = if SIZE.is_power_of_two() {
        ()
    } else {
        panic!("SIZE must be a power of two")
    };

    pub const fn new() -> Self {
        Self {
            buf: unsafe { core::mem::MaybeUninit::zeroed().assume_init() },
            reader: 0,
            writer: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.reader == self.writer
    }

    pub fn is_full(&self) -> bool {
        self.next_writer() + 1 == self.reader
    }

    pub fn len(&self) -> usize {
        unimplemented!()
    }

    pub fn push(&mut self, val: T) -> Result<(), T> {
        if let Some(write) = self.step_writer(val) {
            Err(write)
        } else {
            Ok(())
        }
    }

    pub fn pop(&mut self) -> Result<T, ()> {
        if let Some(read) = self.step_reader() {
            Ok(read)
        } else {
            Err(())
        }
    }

    /// returns Some if value was not successfully pushed.
    fn step_writer(&mut self, write: T) -> Option<T> {
        if self.is_full() {
            Some(write)
        } else {
            self.buf[self.writer] = ManuallyDrop::new(write);
            self.writer = self.next_writer();
            None
        }
    }

    fn step_reader(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            let index = self.reader;
            self.reader = self.next_reader();
            Some(unsafe { ManuallyDrop::take(&mut self.buf[index]) })
        }
    }

    fn next_writer(&self) -> usize {
        (self.writer + 1) % SIZE
    }

    fn next_reader(&self) -> usize {
        (self.reader + 1) % SIZE
    }
}

impl<T, const SIZE: usize> Drop for RingBuf<T, SIZE> {
    fn drop(&mut self) {
        while let Some(read) = self.step_reader() {
            drop(read)
        }
    }
}
