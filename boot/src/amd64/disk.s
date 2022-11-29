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
