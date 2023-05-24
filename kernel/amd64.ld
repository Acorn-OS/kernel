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

    __section_text_start = .;

    .text : {
        KEEP(*(.text.boot))
        *(.text .text.*)
    } :text

    __section_text_end = .;

    . += 0x1000;

    __section_r_start = .;

    .rodata : {
        *(.rodata .rodata.*)
    } :rodata

    .init_array : {
        __init_array_beg = .;
        KEEP(*(.init_array* .init_array.*))
        __init_array_end = .;
    } :rodata 

    __section_r_end = .;

    . += 0x1000;

    __section_rw_start = .;

    .data : {
        *(.data .data.*)
    } :data

    .limine_reqs : {
        *(.limine_reqs)
    } :data

    .bss : {
        *(COMMON)
        *(.bss .bss.*)
    } :data

    __section_rw_end = .;
}