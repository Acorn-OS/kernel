use crate::arch::fb;
use crate::util::locked::ThreadLocked;

pub const WIDTH: usize = fb::WIDTH;
pub const HEIGHT: usize = fb::HEIGHT;

pub struct Cursor {
    pub x: usize,
    pub y: usize,
}

struct LockedWriter;

static WRITER: ThreadLocked<LockedWriter> = ThreadLocked::new(LockedWriter);

static mut CURSOR: Cursor = Cursor { x: 0, y: 0 };

impl Cursor {
    fn map_to_arch(&self) -> fb::Cursor {
        fb::Cursor {
            x: self.x,
            y: self.y,
        }
    }
}

impl LockedWriter {
    fn putb(&self, b: u8) {
        fb::putb(unsafe { CURSOR.map_to_arch() }, b);
        unsafe { step_cursor() };
    }

    fn putc(&self, c: char) {
        self.putb(c as u8)
    }

    fn puts(&self, s: &str) {
        for c in s.chars() {
            self.putc(c)
        }
    }
}

unsafe fn step_cursor() {
    CURSOR.x += 1;
    if CURSOR.x > WIDTH {
        CURSOR.x = 0;
        CURSOR.y += 1;
        if CURSOR.y > HEIGHT {
            CURSOR.y = 0;
        }
    }
}

pub fn putb(b: u8) {
    WRITER.lock().putb(b)
}

pub fn putc(c: char) {
    WRITER.lock().putc(c);
}

pub fn puts(s: &str) {
    WRITER.lock().puts(s);
}
