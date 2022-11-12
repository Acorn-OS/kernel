# Enables A20 line and copies the rest of the bootstrapping code.

.section .boot.start, "awx"
.code16

.global _start
_start:
    # Allocated 30 KiB of stack memory.  
    mov sp, 0x7C00

    # Pushes the new CS to the stack and pops it!
    mov eax, offset .Lreset_cs
    push 0x00
    push eax
    retf 
.Lreset_cs:
    
    # Zeroes the segment registers.
    xor ax, ax  
    mov ds, ax
    mov ss, ax
    mov es, ax
    mov gs, ax

    # Enable A20 line lazily.
    in al, 0x92
    test al, 2
    jnz .La20_enabled
    or al, 2
    and al, 0xFE
    out 0x92, al
.La20_enabled:

    # Load in GDT table and enter protected mode temporarily.
    cli         # Disallow interrupts to occur duirng mode shift.
    push ds     # Saves the current data segment on the stack
    
    lgdt[gdt32_ptr] # Load the gdt table to enter PM.

    # Enable PM bit.
    mov eax, cr0
    or ax, 1 
    mov cr0, eax

    jmp .Lpm # Enter PM. Since cs is set to zero, we're still executing real mode code! 
.Lpm:
    # Set segment descriptor for data to allow higher than a megabyte of addressable memory.
    mov bx, 0x10
    mov ds, bx

    # Now get back to real mode.
    and al, 0xFE
    mov cr0, eax

    pop ds  # Reesore the old data segment.

    # Now we're able to address memory with the eax register and completely avoid segmentation 
    # as well as address > 1 MiB.

    # Now to test if the processor supports extended LBA.
    mov ah, 0x41
    mov bx, 0x55AA
    mov dl, 0x80 
    int 0x13
    mov si, offset msg_no_lba
    jc error

    # Load the rest of the bootstrap code into memory.
    # The bootstrap code is only a part of the bootloader itself.
    # Later stages will load the rest of the bootloader.
    mov si, offset bootstrap_load_adr
    mov lba_load_buffer, si
    mov si, offset bootstrap_load_size
    shr si, 9
    mov lba_packet_size, si 

    mov si, offset lba_struct
    mov ah, 0x42
    mov dl, 0x80
    int 0x13
    mov si, offset msg_ld
    jc error

    # Enter stage2.
    jmp stage2 

    # Infinitely halt execution.
    mov si, offset msg_halt
error:
    call println16
halt:
    cli
    hlt
    jmp halt

# Utility functions.

# Prints string located at si, terminates on the '\0' (0) character.
print16:
    lodsb
    or al, al
    jz .Ldone
    mov ah, 0x0E
    mov bh, 0
    int 0x10
    jmp print16
.Ldone: 
    ret

# Prints string located at si, and then inserts a newline.
println16:
    call print16
    mov si, offset str_newline
    call print16
    ret

# Messages and newline string.

msg_no_lba: .asciz "BIOS doesn't support extended LBA functionality."
msg_halt:   .asciz "halting execution of bootloader..."
msg_bl_zs:  .asciz "Bootloader has a size of zero."
msg_ld:     .asciz "Failed to load the rest of the bootloader into memory."
str_newline:            .byte 10, 13, 0

lba_struct:
    .byte 16
    .byte 0
lba_packet_size:
    .word 1
lba_load_buffer:
    .long 0
lba_starting_lba: 
    .quad 1

# gdt table to allow for addressing higher than the first megabyte in real mode.
gdt32_ptr:
    .word gdt32_end - gdt32 - 1
    .quad gdt32 

gdt32:
.L32null:
    .quad 0
.L32code:
    .word 0xFFFF
    .word 0
    .byte 0
    .byte GDT_PRESENT | GDT_NOT_SYS | GDT_EXEC | GDT_RW
    .byte GDT_GRAN_4K | GDT_SZ_32 | 0xF
    .byte 0
.L32data:
    .word 0xFFFF
    .word 0
    .byte 0
    .byte GDT_PRESENT | GDT_NOT_SYS | GDT_RW
    .byte GDT_GRAN_4K | GDT_SZ_32 | 0xF
    .byte 0
gdt32_end:

.org 0x1B8

.org 510
.byte 0x55, 0xAA