use crate::arch::vm;
use crate::mm::vmm::{Flags, VirtualMemory};
use crate::mm::{heap, pmm};
use crate::process::thread::Thread;
use crate::process::{self, Process, ProcessId};
use crate::util::adr::VirtAdr;
use core::ptr::NonNull;
use elf::Elf64;

pub fn spawn(elf: &Elf64) -> (NonNull<Process>, ProcessId) {
    let mut vmm = unsafe { map(elf, VirtualMemory::new_userland()) };
    let thread = Thread::new(&mut vmm, elf.program_entry(), 256);
    process::new_kernel_proc(heap::alloc(vmm), thread)
}

unsafe fn map(elf: &Elf64, mut vmm: VirtualMemory) -> VirtualMemory {
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
                    Some(VirtAdr::new(align_floor!(vadr, pmm::PAGE_SIZE as u64))),
                    1,
                    Flags::Phys {
                        flags: vm::Flags::PRESENT | vm::Flags::RW | vm::Flags::USER,
                        phys: alloc.phys(),
                    },
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
