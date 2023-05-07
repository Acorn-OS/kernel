use crate::process::scheduler;
use super::super::apic::lapic;
use super::StackFrame;

#[no_mangle]
unsafe extern "C" fn irq_timer(stackframe: *mut StackFrame) -> *mut StackFrame {
    let new_stackframe = scheduler::step(stackframe);
    lapic::eoi();
    new_stackframe
}
