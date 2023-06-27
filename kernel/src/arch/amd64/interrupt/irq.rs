use super::super::apic::lapic;
use super::StackFrame;
use crate::process::thread::sched;

#[no_mangle]
unsafe extern "C" fn irq_timer(stackframe: *mut StackFrame) {
    sched::step(stackframe);
    lapic::eoi();
}
