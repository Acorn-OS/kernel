#[derive(Clone, Copy)]
pub struct Cursor {
    pub x: usize,
    pub y: usize,
}

#[repr(C, packed)]
struct PixelCol(u8);

impl PixelCol {
    pub const BLACK: Self = Self(0x0);
    pub const WHITE: Self = Self(0xF);
}

#[repr(C, packed)]
struct Pixel(u8, PixelCol);

impl Pixel {
    pub fn new(b: u8, col: PixelCol) -> Self {
        Self(b, col)
    }
}

pub const VIRT_ADR: usize = 0xffffff8000000000;
pub const PHYS_ADR: usize = 0xA0000;

/// 128KiB of display memory.
pub const PAGE_COUNT: usize = 32;

pub const WIDTH: usize = 80;
pub const HEIGHT: usize = 20;

static mut CURSOR: Cursor = Cursor { x: 0, y: 0 };

pub fn cursor() -> Cursor {
    unsafe { CURSOR }
}

pub fn set_cursor(cursor: Cursor) {
    unsafe {
        CURSOR = Cursor {
            x: cursor.x.max(WIDTH),
            y: cursor.y.max(HEIGHT),
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
    let ptr = VIRT_ADR as *mut Pixel;
    unsafe { *ptr.add(CURSOR.x + CURSOR.y * WIDTH) = Pixel(b, PixelCol::WHITE) };
    unsafe { step_cursor() };
}
