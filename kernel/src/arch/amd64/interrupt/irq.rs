use super::super::apic::lapic;
use super::StackFrame;
use crate::scheduler;

#[no_mangle]
unsafe extern "C" fn irq_timer(stackframe: *mut StackFrame) {
    scheduler::step(stackframe);
    lapic::eoi();
}
