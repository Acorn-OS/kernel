use crate::arch::fb;

pub fn run() {
    fb::clear();
    fb::putlns("hello world!\n");
    fb::putlns("hello son!");
}
