.section .boot.text, "awx"
.code16

stage2:
    # Clears the current framebuffer.
    call clear_fb

    # Prints hello message.
    mov si, offset msg_hello
    call println16

    # Load memory map into buffer.
    # TODO!

    # Loads the kernel into memory
    mov edi, offset kernel_load_adr # Address to load into.
    mov esi, offset kernel_rom_adr  # LBA address to load from.
    mov ecx, offset kernel_size     # Amount of sectors to load.
    shr esi, 9
    shr ecx, 9
    call load_from_disk
    mov si, offset load_error_msg
    jc error

    # Turn off cursor for the framebuffer.
    mov ah, 1
    mov ch, 0x3F
    int 0x10

    # Enter protected mode for real this time and enter stage 3!
    lgdt[gdt32_ptr] 
    mov eax, cr0
    or ax, 1 
    mov cr0, eax

    # Set segment registers for PM.
    mov bx, 0x10 
    mov ds, bx 
    mov ss, bx 
    mov es, bx 
    mov gs, bx 

    # Make a far return to enter into PM at stage3 address.
    mov eax, offset stage3
    push 0x08
    push eax
    retf 

    mov si, msg_halt
    jmp error

.align 512
disk_load_buffer:
    .space 512

clear_fb:
    mov al, ' '
    mov ah, 0x07
    mov edi, 0xB8000
    mov cx, 0x4000
    rep stosw [edi]
    mov ah, 0x02
    xor bx, bx
    xor dx, dx
    int 0x10
    ret

msg_hello:  .asciz "Acorn OS"
load_error_msg: .asciz "loading kernel into memory failed."