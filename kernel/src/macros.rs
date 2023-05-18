#![allow(unused)]

macro_rules! is_aligned {
    ($expr:expr, $align:expr) => {{
        ($expr) & ($align - 1) == 0
    }};
}

macro_rules! align_floor {
    ($expr:expr, $align:expr) => {{
        ($expr).div_floor($align) * ($align)
    }};
}

macro_rules! align_ceil {
    ($expr:expr, $align:expr) => {{
        ($expr).div_ceil($align) * ($align)
    }};
}

macro_rules! pages {
    ($expr:expr) => {{
        let val = ($expr) as usize;
        (val).div_ceil($crate::mm::pmm::PAGE_SIZE)
    }};
}
