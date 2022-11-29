ENTRY(_start)
OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)

SECTIONS {
    kvma_beg = 0xFFFFFFFF80000000;
    kvma_end = 0xFFFFFFFFFFFFFFFF;
    kernel_load_adr = 0x01000000;
    kernel_rom_adr = kernel_load_adr;


    . = 0x7C00;
    .boot.start : AT(0) {
        KEEP(*(.boot.start))
        . = ALIGN(512);
    }

    .boot.text ALIGN(512) : AT(ADDR(.boot.text) - 0x7C00) {
        bootstrap_load_adr = .;
        *(.boot.text .boot.text.*)
        . = ALIGN(512);
        bootstrap_load_size = SIZEOF(.boot.text);
    }

    .boot.bss ALIGN(512) (NOLOAD) : {
        boot_bss_start = .;
        *(.boot.bss .boot.bss.*)
        . = ALIGN(512);
        boot_bss_end = .;
    }

    
    . = kvma_beg + kernel_load_adr;

    __executable_start = .;
    .setup : AT(ADDR(.setup) - kvma_beg) {
        KEEP(*(.setup))
    }

    .text ALIGN(4K) : AT(ADDR(.text) - kvma_beg) {
        *(.text .text.*)
    }

    .data ALIGN(4K) : AT(ADDR(.data) - kvma_beg) {
        *(.data .data.*)
    }

    .misc ALIGN(4K) : AT(ADDR(.misc) - kvma_beg) {
        *(.eh_frame)
        *(.eh_frame_hdr)
    }

    .rodata ALIGN(4K) : AT(ADDR(.rodata) - kvma_beg) {
        *(.rodata .rodata.*)
    }

    .eh_frame ALIGN(4K) : AT(ADDR(.eh_frame) - kvma_beg) { 
        PROVIDE(__eh_frame = .);
        KEEP(*(.eh_frame)) 
        *(.eh_frame.*) 
    }

    . = ALIGN(512);

    __etext = .;
    kernel_size = . - (kvma_beg + kernel_load_adr);

    .bss ALIGN(4K) (NOLOAD) : {
        bss_start = .;
        . += 4M;
		kernel_stack_top = .;
        *(COMMON)
        *(.bss .bss.*)
        bss_end = .;
    }

    . = . - kvma_beg;

    . = ALIGN(4K);
    base_page_table = .;
    . += 4K;

    kwm_beg = .;
    . += 256M;
    kwm_end = .;
} 

