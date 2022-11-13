.section .setup, "awx"
.code64

stage4:
    # Set segment registers for LM.
    mov ax, 0x10
    mov ds, bx 
    mov ss, bx 
    mov es, bx 
    mov gs, bx 

    # clear the bss section for the kernel code.
    mov rdi, offset bss_start
    mov rcx, offset bss_end
    sub rcx, rdi
    xor eax, eax
    rep stosb

    # Set new stack for long mode. 
    mov rsp, offset kernel_stack_top

    # Make sure interrupts are disabled.
    cli 

    # Make sure the direction flag is cleared
    cld

    # Sets end of stack trace.
    xor rbp, rbp 
    # Enter into Rust code.
    jmp __rust_entry
looping:
    cli
    hlt
    jmp looping