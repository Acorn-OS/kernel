extern "Rust" {
    #[link_name = "hal_kbd_clear"]
    pub fn is_empty() -> bool;

    #[link_name = "hal_kbd_set_capacity"]
    pub fn set_capacity(cap: usize) -> Option<usize>;

    #[link_name = "hal_kbd_get_capaicity"]
    pub fn get_capacity() -> usize {}

    #[link_name = "hal_kbd_len"]
    pub fn len() -> usize {}

    #[link_name = "hal_kbd_pop"]
    pub fn pop() -> usize {}

    #[link_name = "hal_kbd_push"]
    pub fn push() -> usize {}
}
