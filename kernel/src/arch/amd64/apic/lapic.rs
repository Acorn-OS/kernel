use crate::mm::pmm;

use super::super::{cpuc, msr};

const BASE_LAPIC_MSR: u32 = 0x1b;

fn get_local_apic_base_adr() -> u64 {
    msr::get(BASE_LAPIC_MSR) & 0xffffff000
}

fn set_local_apic_base_adr(_v: u64) {
    unimplemented!()
}

fn toggle_enabled_local_apic(v: bool) {
    let mut apic = msr::get(BASE_LAPIC_MSR);
    btoggle!(apic, 11, v);
    msr::set(BASE_LAPIC_MSR, apic)
}

#[derive(Clone, Copy)]
pub struct LApicPtr(u64);

impl LApicPtr {
    pub const ID: u16 = 0x20;
    pub const VER: u16 = 0x30;
    pub const TASK_PRIO: u16 = 0x80;
    pub const ARBITRATION_PRIO: u16 = 0x90;
    pub const PROCCESOR_PRIO: u16 = 0xa0;
    pub const EOI: u16 = 0xb0;
    pub const REMOTE_READ: u16 = 0xC0;
    pub const LOGICAL_DST: u16 = 0xd0;
    pub const DST_FMT: u16 = 0xe0;
    pub const SPURIUOS_INT_VEC: u16 = 0xf0;
    pub const ERROR_STATUS: u16 = 0x280;
    pub const LVT_CORRECTED_MACHINE_CHECK_INT: u16 = 0x2f0;
    pub const LVT_TIMER: u16 = 0x320;
    pub const LVT_THERMAL: u16 = 0x330;
    pub const LVT_PERF_MONITORING_COUNTERS: u16 = 0x340;
    pub const LVT_LINT0: u16 = 0x350;
    pub const LVT_LINT1: u16 = 0x360;
    pub const LVT_ERROR: u16 = 0x370;
    pub const INIT_CNT: u16 = 0x380;
    pub const CUR_CNT: u16 = 0x390;
    pub const DIV_CONF: u16 = 0x3e0;

    pub unsafe fn write_reg(&self, reg: u16, val: u32) {
        debug_assert!(reg < 0x400);
        let reg = reg & 0x0fff;
        let ptr = (self.0 + reg as u64) as *mut u32;
        *ptr = val
    }

    pub unsafe fn read_reg(&self, reg: u16) -> u32 {
        debug_assert!(reg < 0x400);
        let reg = reg & 0x0fff;
        let ptr = (self.0 + reg as u64) as *const u32;
        *ptr
    }

    pub unsafe fn eoi(self) {
        self.write_reg(LApicPtr::EOI, 0);
    }
}

#[must_use]
pub unsafe fn create_local() -> LApicPtr {
    let phys_adr = get_local_apic_base_adr();
    let virt = pmm::phys_to_hhdm(phys_adr);
    toggle_enabled_local_apic(true);
    let ptr = LApicPtr(virt as u64);
    // enables the lapic.
    ptr.write_reg(
        LApicPtr::SPURIUOS_INT_VEC,
        ptr.read_reg(LApicPtr::SPURIUOS_INT_VEC) | 0x1FF,
    );
    ptr.write_reg(LApicPtr::INIT_CNT, 0x8000000);
    ptr.write_reg(LApicPtr::DIV_CONF, 0b1011);
    ptr.write_reg(LApicPtr::LVT_TIMER, 0x20 | (0b01 << 17));
    ptr
}

pub unsafe fn eoi() {
    let ptr = cpuc::get_kernel();
    debug_assert!(!ptr.is_some(), "cpuc ptr is null");
    let mut ptr = ptr.unwrap_unchecked();
    ptr.as_mut().lapic_ptr.eoi();
}
