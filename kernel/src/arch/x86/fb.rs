use core::mem::size_of;

const FB_PTR: *mut Pixel = 0xB8000 as *mut Pixel;
const FB_WIDTH: usize = 80;
const FB_HEIGHT: usize = 25;

static mut X: usize = 0;
static mut Y: usize = 0;

#[derive(Clone, Copy)]
pub struct Color(u8);
assert_eq_size!(Color, u8);

impl Color {
    pub const BLACK: Self = Color(0);
    pub const WHITE: Self = Color(15);
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct Pixel(u8, Color);
assert_eq_size!(Pixel, u16);

#[inline(always)]
unsafe fn newline() {
    Y += 1;
    if Y >= FB_HEIGHT {
        Y = 0;
    }
}

#[inline(always)]
unsafe fn step() {
    X += 1;
    if X >= FB_WIDTH {
        X = 0;
        newline();
    }
}

#[inline(always)]
fn place(c: char, col: Color) {
    unsafe { FB_PTR.add(X).add(Y * FB_WIDTH).write(Pixel(c as u8, col)) };
}

pub unsafe fn init() {
    clear();
}

pub fn set_pos(x: usize, y: usize) {
    unsafe {
        if x >= FB_WIDTH || y >= FB_HEIGHT {
            warn!("Attempting to set an out-of-bounds position for framebuffer access ({x}, {y}) where bounds are ({FB_WIDTH}, {FB_HEIGHT}).")
        } else {
            X = x;
            Y = y;
        }
    }
}

#[inline]
pub fn putc(c: char, col: Color) {
    unsafe {
        match c {
            '\n' => newline(),
            '\r' => X = 0,
            c => {
                place(c, col);
                step();
            }
        }
    }
}

#[inline]
pub fn puts(s: &str, col: Color) {
    for c in s.chars() {
        putc(c, col)
    }
}

#[inline]
pub fn clear() {
    unsafe {
        FB_PTR.write_bytes(0, FB_WIDTH * FB_HEIGHT * size_of::<Pixel>());
        X = 0;
        Y = 0;
    }
}
