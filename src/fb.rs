use crate::arch::fb;

proc_macro::idef! {
    static TTY = {
        pos_x: usize = 0,
        pos_y: usize = 0,
    }
    impl {
        pub fn clear(&mut self) {
            unsafe { fb::clear() }
        }

        pub fn set_pos(&mut self, x: usize, y: usize) {
            unsafe { fb::set_pos(x, y) }
        }

        pub fn putc(&mut self, c: char) {
            unsafe { fb::putc(c, fb::Colour::WHITE) }
        }

        pub fn puts(&mut self, s: &str) {
            unsafe { fb::puts(s, fb::Colour::WHITE) }
        }
    }
}
