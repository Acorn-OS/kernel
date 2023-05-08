use core::ptr::addr_of;

macro_rules! def_symbol {
    ($name:ident: $sym:ident) => {
        pub fn $name() -> u64 {
            extern "C" {
                static $sym: u8;
            }
            unsafe { addr_of!($sym) as u64 }
        }
    };
}

def_symbol!(section_r_start: __section_r_start);
def_symbol!(section_r_end: __section_r_end);

def_symbol!(section_rw_start: __section_rw_start);
def_symbol!(section_rw_end: __section_rw_end);

def_symbol!(section_text_start: __section_text_start);
def_symbol!(section_text_end: __section_text_end);
