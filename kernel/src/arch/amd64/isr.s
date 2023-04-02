.code64
.section .text

.set EXCEPTION_DUMMY_ERROR, 0xFFFFFFFFFFFFFFFF

.macro PUSH_REGS
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
.endm

.macro POP_REGS
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
.endm

.macro EXCPT n, name 
	.align 8
	.global irq_handler_\n
	irq_handler_\n:
        cld
		push \n
		PUSH_REGS
		mov rdi, rsp 
		.extern excpt_\name
		call excpt_\name
		mov rsp, rax 
		POP_REGS
		add rsp, 16 
		iretq
.endm 

.macro EXCPT_DUMMY n, name 
	.align 8
	.global irq_handler_\n
	irq_handler_\n:
        cld
        push EXCEPTION_DUMMY_ERROR 
		push \n
		PUSH_REGS
		mov rdi, rsp 
		.extern excpt_\name
		call excpt_\name
		mov rsp, rax 
		POP_REGS
		add rsp, 16 
		iretq
.endm 

.macro IRQ n, name 
	.align 8
	.global irq_handler_\n
	irq_handler_\n:
        cld
        push EXCEPTION_DUMMY_ERROR
		push \n 
		PUSH_REGS
		mov rdi, rsp 
		.extern \name
		call \name
		mov rsp, rax 
		POP_REGS
		add rsp, 16
		iretq
.endm 

EXCPT_DUMMY 0 division_error
EXCPT_DUMMY 1 debug
EXCPT_DUMMY 2 non_maskable_interrupt
EXCPT_DUMMY 3 breakpoint
EXCPT_DUMMY 4 overflow
EXCPT_DUMMY 5 bound_range_exceeded
EXCPT_DUMMY 6 invalid_opcode
EXCPT_DUMMY 7 device_not_available
EXCPT 8 double_fault
EXCPT_DUMMY 9 deprecated
EXCPT 10 invalid_tss
EXCPT 11 segment_not_present
EXCPT 12 stack_segment_fault
EXCPT 13 general_protection_fault
EXCPT 14 page_fault
EXCPT_DUMMY 15 reserved
EXCPT_DUMMY 16 x87_floating_point
EXCPT 17 alignment_check
EXCPT_DUMMY 18 machine_check
EXCPT_DUMMY 19 simd_floating_point
EXCPT_DUMMY 20 virtualization
EXCPT 21 control_protection
EXCPT_DUMMY 22 reserved
EXCPT_DUMMY 23 reserved 
EXCPT_DUMMY 24 reserved 
EXCPT_DUMMY 25 reserved 
EXCPT_DUMMY 26 reserved
EXCPT_DUMMY 27 reserved
EXCPT_DUMMY 28 hypervisor_injection
EXCPT 29 vmm_communication
EXCPT 30 security
EXCPT_DUMMY 31 reserved

IRQ 32 irq_timer
IRQ 33 unimp 
IRQ 34 unimp
IRQ 35 unimp
IRQ 36 unimp
IRQ 37 unimp
IRQ 38 unimp
IRQ 39 unimp
IRQ 40 unimp
IRQ 41 unimp
IRQ 42 unimp
IRQ 43 unimp
IRQ 44 unimp
IRQ 45 unimp 
IRQ 46 unimp
IRQ 47 unimp

