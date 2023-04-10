use super::msr;
use crate::mm::vmm::VirtualMemory;

const BASE_LAPIC_MSR: u32 = 0x1b;

fn get_local_apic_base_adr() -> u64 {
    msr::get(BASE_LAPIC_MSR) & 0xffffff000
}

fn set_local_apic_base_adr(_v: u64) {
    todo!()
}

fn toggle_enabled_local_apic(v: bool) {
    let mut apic = msr::get(BASE_LAPIC_MSR);
    btoggle!(apic, 11, v);
    msr::set(BASE_LAPIC_MSR, apic)
}

#[derive(Clone, Copy)]
pub struct LApicPtr(u64);

impl LApicPtr {
    const ID: u16 = 0x20;
    const VER: u16 = 0x30;
    const TASK_PRIO: u16 = 0x80;
    const ARBITRATION_PRIO: u16 = 0x90;
    const PROCCESOR_PRIO: u16 = 0xa0;
    const EOI: u16 = 0xb0;
    const REMOTE_READ: u16 = 0xC0;
    const LOGICAL_DST: u16 = 0xd0;
    const DST_FMT: u16 = 0xe0;
    const SPURIUOS_INT_VEC: u16 = 0xf0;
    const ERROR_STATUS: u16 = 0x280;
    const LVT_CORRECTED_MACHINE_CHECK_INT: u16 = 0x2f0;
    const LVT_TIMER: u16 = 0x320;
    const LVT_THERMAL: u16 = 0x330;
    const LVT_PERF_MONITORING_COUNTERS: u16 = 0x340;
    const LVT_LINT0: u16 = 0x350;
    const LVT_LINT1: u16 = 0x360;
    const LVT_ERROR: u16 = 0x370;
    const INIT_CNT: u16 = 0x380;
    const CUR_CNT: u16 = 0x390;
    const DIV_CONF: u16 = 0x3e0;

    unsafe fn write_reg(&self, reg: u16, val: u32) {
        debug_assert!(reg < 0x400);
        let reg = reg & 0x0fff;
        let ptr = (self.0 + reg as u64) as *mut u32;
        *ptr = val
    }

    unsafe fn read_reg(&self, reg: u16) -> u32 {
        debug_assert!(reg < 0x400);
        let reg = reg & 0x0fff;
        let ptr = (self.0 + reg as u64) as *const u32;
        *ptr
    }

    pub unsafe fn eoi(self) {
        self.write_reg(LApicPtr::EOI, 0);
    }
}

pub unsafe fn create_local(page_map: &mut VirtualMemory) -> LApicPtr {
    let phys_adr = get_local_apic_base_adr();
    let virt = page_map.map_pages(1, phys_adr);
    toggle_enabled_local_apic(true);
    let ptr = LApicPtr(virt as u64);
    // enables the lapic.
    ptr.write_reg(
        LApicPtr::SPURIUOS_INT_VEC,
        ptr.read_reg(LApicPtr::SPURIUOS_INT_VEC) | 0x1FF,
    );
    ptr.write_reg(LApicPtr::INIT_CNT, 0x100000);
    ptr.write_reg(LApicPtr::DIV_CONF, 0b1011);
    ptr.write_reg(
        LApicPtr::LVT_TIMER,
        0x20 | (0b01 << 17) | (1 << 16), /* mask the irq */
    );
    ptr
}
