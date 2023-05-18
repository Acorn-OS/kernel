.section .bss
.align 16
stack_beg:
    .space (64 << 10), 0
stack_end:

.code64
.section .text
.extern kernel_early
.global _start 
_start:
    mov rsp, offset stack_end 
    push 0
    mov rbp, 0
    push rbp
    call kernel_early
hang:
    hlt
    jmp hang