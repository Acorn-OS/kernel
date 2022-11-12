#[derive(Clone, Copy)]
pub enum Color {
    Black,
    White,
}

extern "Rust" {
    #[link_name = "hal_fb_clear"]
    pub fn clear();

    #[link_name = "hal_fb_set_pos"]
    pub fn set_pos(x: usize, y: usize);

    #[link_name = "hal_fb_putc"]
    pub fn putc(c: char);

    #[link_name = "hal_fb_puts"]
    pub fn puts(s: &str);

    #[link_name = "hal_fb_set_color"]
    pub fn set_color(color: Color);

}
