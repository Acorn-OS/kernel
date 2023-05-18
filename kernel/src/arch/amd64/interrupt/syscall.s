.section .bss
.align 16
    .space (1 << 12), 0
kernel_stack:

.section .data
userspace_stack_save: .quad 0
kernel_stack_save: .quad kernel_stack

.code64
.section .text
.extern syscall_handler
.global syscall_enter
syscall_enter:
    swapgs 

    mov [userspace_stack_save], rsp
    mov rsp, [kernel_stack_save]

    push rcx 
    push r11
    push rbp 

    call syscall_handler
    
    pop rbp 
    pop r11 
    pop rcx

    mov rsp, [userspace_stack_save]
    
    swapgs 

    sysretq