bitfield! {
    struct Redirection(u64) {
        vec: u8 @ 0..=7,
        deiliver_mode: u8 @ 8..=10,
        destination: bool @ 11,
        busy: bool @ 12,
        polarity: bool @ 13,
        level_triggered: bool @ 14,
        trigger_mode: bool @ 15,
        mask: bool @ 16,
        dest: u8 @ 56..63,
    }
}

impl Redirection {
    const DELIVERY_MODE_NORMAL: u8 = 0;
    const DELIVERY_MODE_LOW_PRIORITY: u8 = 1;
    const DELIVERY_MODE_SYSTEM_MANAGEMENT: u8 = 2;
    const DELIVERY_MODE_NON_MASKABLE: u8 = 4;
    const DELIVERY_MODE_INIT: u8 = 5;
    const DELIVERY_MODE_EXTERNAL: u8 = 7;
    const POLARITY_HIGH: bool = false;
    const POLARITY_LOW: bool = true;
    const TRIGGER_MODE_EDGE_SENSITIVE: bool = false;
    const TRIGGER_MODE_LEVEL_SENSITIVE: bool = true;
}

struct IoApicPtr(u64);

impl IoApicPtr {}

pub unsafe fn init() {
    trace!("initializing the IO-APIC");
}
