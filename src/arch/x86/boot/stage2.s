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
    mov edi, offset __kernel_load_adr # Address to load into.
    mov esi, offset __kernel_rom_adr  # LBA address to load from.
    mov ecx, offset __kernel_size     # Amount of sectors to load.
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

.section .boot.text, "awx"
.code16

# Loads sectors from the disk into memory.
# EDI:  address to load into.
# ESI:  LBA address to load from.
# ECX:  amount of sectors to load.
# Carry flag is set to 0 on success.
load_from_disk:
    cld
    repeat:
        # Setup the transfer struct.
        mov lba, esi

        # Saves ESI and CX.
        push esi 
        push ecx

        # Perform the transfer.
        mov si, offset da_pack
        mov ah, 0x42
        mov dl, 0x80        
        int 0x13
        jc on_fail

        # Transfer onto destination.
        mov esi, offset buf
        mov ecx, (512 / 4)
        rep movsd [edi], [esi]

        # Restores ESI and CX.
        pop ecx
        pop esi

        # ESI now points to the new sector in the disk
        inc esi

        loop repeat
    clc
    jmp on_success
on_fail:
    pop ecx
    pop esi
on_success:    
    ret

.align 2
buf:
    .space 512

da_pack:
    # packet size.
    .byte 16
    # always zero.
    .byte 0
    # amount of sectors to transfer.
    .word 1
    # unused (default value).
    .word buf
    .word 0
lba:
    # lba address.    
    .quad 0

msg_hello:  .asciz "Acorn OS"
load_error_msg: .asciz "loading kernel into memory failed."