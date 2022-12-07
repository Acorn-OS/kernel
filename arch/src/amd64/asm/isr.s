.code64

.macro irq handler index
.align 8
.global _irq_handler_\index 
_irq_handler_\index:
    push \index
    push rax
    push rbx
    push rcx
    push rdx
    push rsi
    push rdi
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15
    push rbp
    cld
    mov rdi, rsp
    .extern \handler
    call \handler
    mov rsp, rax
    pop rbp
    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop rbx
    pop rax
    add rsp, 8
    iretq
.endm

.macro except handler index 
.align 8
.global _except_handler_\index
_except_handler_\index:
    push \index
    push rax
    push rbx
    push rcx
    push rdx
    push rsi
    push rdi
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15
    push rbp
    cld
    mov rdi, rsp
    .extern \handler
    call \handler
    mov rsp, rax
    pop rbp
    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop rbx
    pop rax
    add rsp, 16
    iretq
.endm

irq programmable_interrupt_timer 0
irq keyboard 1
irq cascade 2
irq com2 3
irq com1 4
irq lpt2 5
irq floppy_disk 6 
irq lpt1 7 
irq cmos_rtc 8
irq free0 9
irq free1 10
irq free2 11
irq ps2_mouse 12
irq coprocessor 13;
irq primary_ata_disk 14
irq secondary_ata_disk 15

except divide_by_zero 0
except debug 1
except non_maskable_interrupt 2
except breakpoint 3
except overflow 4
except bound_range_exceeded 5
except invalid_opcode 6
except device_not_available 7
except double_fault 8
except coprocessor_segment_overrun 9
except invalid_tss 10
except segment_not_present 11
except stack_segment_fault 12
except general_protection_fault 13
except page_fault 14
except reserved_15 15
except floating_point_exception_x87 16
except alignent_check 17
except machine_check 18
except floating_point_exception_simd 19
except virtualization_exception 20
except control_protection_exception 21
except reserved_22 22
except reservec_23 23
except reserved_24 24
except reserved_25 25
except reserved_26 26
except reserved_27 27
except hypervisor_injection_exception 28
except vmm_communication_exception 29
except security_exception 30
except reserved_31 31