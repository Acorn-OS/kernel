static mut CUR: usize = usize::MAX;

const KWM_LEN: usize = 4096 * 4097;
#[repr(align(4096))]
struct WM([u8; KWM_LEN]);

static mut KWM: WM = WM([0; KWM_LEN]);

pub unsafe fn init() {
    /* TODO: remove constant */
    unsafe { CUR = KWM.0.as_ptr() as usize };
}

pub unsafe fn reserve_amount(amount: usize) -> *mut u8 {
    if CUR.saturating_add(amount) < /* TODO: remove constant */ KWM.0.as_ptr() as usize + KWM_LEN {
        let ptr = CUR as *mut u8;
        CUR += amount;
        ptr
    } else {
        panic!("Out of allocatable work memory for the kernel. {amount}")
    }
}
