.section .boot.text, "awx"
.code32

# Setup the kernel page table, and identiy map the first 16MiB.
pg_init:
    # Maps pt4[0] to pd0
    mov edi, offset pt4
    mov eax, offset pd0
    or eax, 0x03
    mov [edi], eax
    
    # Identity maps the first 4GiB.
    mov edi, offset pd0   
    mov long ptr [edi], 0x83
    mov long ptr [edi + 8], 0x83 | (1 << 30)
    mov long ptr [edi + 16], 0x83 | (2 << 30)
    mov long ptr [edi + 24], 0x83 | (3 << 30)

    # Map the first 2 GiB of physical memory to virtual memory [0xffffffff80000000, 0xffffffffffffffff].

    # Maps pt4[511] to pd1 
    mov edi, offset pt4
    mov eax, offset pd1 
    or eax, 0x03
    mov long ptr [edi + (511 * 8)], eax

    # Maps pd1[511] and pd1[510] to the first two GBs
    mov edi, offset pd1
    mov long ptr [edi + (511 * 8)], (1 << 30) | 0x83
    mov long ptr [edi + (510 * 8)], 0x83

    # Prevents OoOE shenanigans from screwing us up.
    wbinvd
    mfence

    # Add top page directory address to the cr3.
    mov eax, offset pt4
    mov cr3, eax

    # Enable the PEA paging bit.
    mov eax, cr4
    or eax, (1 << 5)
    mov cr4, eax

    ret

.align 4096
pt4:
    .space 4096

.align 4096 
pd0:
    .space 4096

.align 4096
pd1:
    .space 4096
