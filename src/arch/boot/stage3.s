.section .boot.text, "awx"
.code32

stage3:
    cli # Clear interrupt flag.
    cld # Clear direction flag.

    # Clear bss section for the boot code.
    xor eax, eax
    mov esi, offset boot_bss_start
    mov ecx, offset boot_bss_end
    sub ecx, esi
    rep stosb

    call vga_clear
    mov esi, offset msg_hello
    call vga_println

    # Check support for features.
    call check_cpuid
    call check_lm_support

    # Initializes paging.
    call pg_init

    jmp enter_stage4

    mov esi, offset msg_halt
error32:
    call vga_println
halt32:
    cli
    hlt
    jmp halt32

check_cpuid:
    pushfd
    pushfd
    xor long ptr [esp], 0x00200000
    popfd
    pushfd
    pop eax
    xor eax, [esp]
    popfd
    and eax, 0x00200000
    jz cpuid_not_found
    ret
cpuid_not_found:
    mov esi, offset msg_no_cpuid_support
    jmp error32

check_lm_support:
    mov eax, 0x80000000
    cpuid
    cmp eax, 0x80000001
    jb lm_support_unavailable
    mov eax, 0x80000001
    cpuid
    test edx, (1 << 29)
    jz lm_support_unavailable
    ret
lm_support_unavailable:
    mov esi, offset msg_no_lm_support
    jmp error32

enter_stage4:
    # Set the LM bit.
    mov ecx, 0xC0000080
    rdmsr
    or eax, (1 << 8)
    wrmsr

    # Enables paging.
    mov eax, cr0
    or eax, (1 << 31)
    mov cr0, eax

    # Enter long mode.
    lgdt [gdt64_ptr]
    mov eax, offset jmp_to_right_place
    push 0x08
    push eax
    retf

    jmp halt32

gdt64_ptr:
    .word gdt64_end - gdt64 - 1
    .quad gdt64 

gdt64:
.L64null:
    .quad 0
.L64code:
    .long 0
    .byte 0
    .byte GDT_PRESENT | GDT_NOT_SYS | GDT_EXEC | GDT_RW
    .byte GDT_GRAN_4K | GDT_LONG_MODE | 0xF
    .byte 0
.L64data:
    .long 0
    .byte 0
    .byte GDT_PRESENT | GDT_NOT_SYS | GDT_RW
    .byte GDT_GRAN_4K | GDT_SZ_32 | 0xF
    .byte 0
.L64tss:
    .long 0x00000068
    .long 0x00CF8900
    .quad 0
gdt64_end:

msg_no_cpuid_support:   .asciz "No cpuid support for target processor."
msg_no_lm_support:      .asciz "No Long Mode support for target processor."

.code64
.section .boot.text

jmp_to_right_place:
    mov rax, offset stage4 
    jmp rax
loop64:
    cli 
    hlt
    jmp loop64