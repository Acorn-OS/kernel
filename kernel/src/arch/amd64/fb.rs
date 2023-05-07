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
assert_eq_size!(Pixel, u16);

impl Pixel {
    pub fn new(b: u8, col: PixelCol) -> Self {
        Self(b, col)
    }
}

pub const PHYS_ADR: usize = 0xb8000;

/// 128KiB of display memory.
pub const PAGE_COUNT: usize = 32;

pub const WIDTH: usize = 80;
pub const HEIGHT: usize = 20;

pub fn putb(pos: Cursor, b: u8) {
    let ptr = PHYS_ADR as *mut Pixel;
    unsafe { *ptr.add(pos.x + pos.y * WIDTH) = Pixel(b, PixelCol::WHITE) };
}
