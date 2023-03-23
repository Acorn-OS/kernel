OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)
ENTRY(_start)

PHDRS
{
    text    PT_LOAD    FLAGS((1 << 0) | (1 << 2)) ; /* Execute + Read */
    rodata  PT_LOAD    FLAGS((1 << 2)) ;            /* Read only */
    data    PT_LOAD    FLAGS((1 << 1) | (1 << 2)) ; /* Write + Read */
}

SECTIONS
{
    . = 0xffffffff80000000;

    .text : {
        *(.text .text.*)
    } :text

    . += 0x1000;

    .rodata : {
        *(.rodata .rodata.*)
    } :rodata

    . += 0x1000;

    .limine_reqs : {
        *(.limine_reqs)
    } :data 

    .data : {
        *(.data .data.*)
    } :data

    .init_array : {
        __init_array_beg = .;
        KEEP(*(.init_array* .init_array.*))
        __init_array_end = .;
    } :data 

    .bss : {
        *(COMMON)
        *(.bss .bss.*)
    } :data
}