use crate::arch::interrupt::StackFrame;
use crate::mm::pmm;
use crate::mm::vmm::{Flags, VMM};
use crate::process::thread::{self, ThreadId};
use crate::process::{self, ProcessId, ProcessPtr, Result};
use crate::util::adr::VirtAdr;
use alloc::boxed::Box;
use elf::Elf64;

pub fn spawn(elf: &Elf64) -> Result<(ProcessPtr, ProcessId)> {
    let mut vmm = unsafe { map(elf, VMM::new_userland()) };
    let stack = unsafe { thread::create_userspace_thread_stack(&mut vmm, 256) };
    let stackframe = Box::new(StackFrame::new_userspace(
        elf.program_entry(),
        stack.adr(),
        vmm.get_page_map(),
    ));
    let (proc, id) = process::new_proc(vmm).expect("failed to create new process");
    unsafe {
        proc.get_mut()
            .add_thread(thread::new(ThreadId::gen(), proc, stackframe)?)?
    };
    Ok((proc, id))
}

unsafe fn map(elf: &Elf64, mut vmm: VMM) -> VMM {
    // load program headers.
    for program_header in elf.program_headers() {
        let mut bytes = program_header.p_memsz;
        let mut vadr = program_header.p_vaddr;
        let mut off = program_header.p_offset;
        let pages = pages!(bytes) as usize;
        for _ in 0..pages {
            let hhdm = if let Some(phys) = vmm.virt_to_phys(VirtAdr::new(vadr)) {
                pmm::phys_to_hhdm(phys)
            } else {
                let alloc = pmm::alloc_pages(1);
                vmm.map(
                    Some(VirtAdr::new(vadr)),
                    1,
                    Flags::PRESENT | Flags::RW | Flags::USER,
                    alloc.phys(),
                );
                alloc.virt()
            };
            let unalignment = (vadr & (pmm::PAGE_SIZE as u64 - 1)) as usize;
            let hhdm = hhdm.ptr();
            let step = (pmm::PAGE_SIZE - unalignment) as u64;
            hhdm.add(unalignment)
                .copy_from(elf.as_ptr().add(off as usize), step as usize);
            vadr += step;
            off += step;
            bytes = bytes.saturating_sub(step);
        }
    }
    vmm
}
