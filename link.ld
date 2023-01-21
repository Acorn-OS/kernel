ENTRY(_start)
OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)

SECTIONS {
    __kvma_beg = 0xFFFFFFFF80000000;
    __kvma_end = 0xFFFFFFFFFFFFFFFF;
    __kernel_load_adr = 0x01000000;
    __kernel_rom_adr = __kernel_load_adr;


    . = 0x7C00;
    .boot.start : AT(0) {
        KEEP(*(.boot.start))
        . = ALIGN(512);
    }

    .boot.text ALIGN(512) : AT(ADDR(.boot.text) - 0x7C00) {
        __bootstrap_load_adr = .;
        *(.boot.text .boot.text.*)
        . = ALIGN(512);
        __bootstrap_load_size = SIZEOF(.boot.text);
    }

    .boot.bss ALIGN(512) (NOLOAD) : {
        __boot_bss_start = .;
        *(.boot.bss .boot.bss.*)
        . = ALIGN(512);
        __boot_bss_end = .;
    }

    
    . = __kvma_beg + __kernel_load_adr;

    __executable_start = .;
    .setup : AT(ADDR(.setup) - __kvma_beg) {
        KEEP(*(.setup))
    }

    .text ALIGN(4K) : AT(ADDR(.text) - __kvma_beg) {
        *(.text .text.*)
    }

    .data ALIGN(4K) : AT(ADDR(.data) - __kvma_beg) {
        *(.data .data.*)
    }

    .misc ALIGN(4K) : AT(ADDR(.misc) - __kvma_beg) {
        *(.eh_frame)
        *(.eh_frame_hdr)
    }

    .rodata ALIGN(4K) : AT(ADDR(.rodata) - __kvma_beg) {
        *(.rodata .rodata.*)
    }

    .eh_frame ALIGN(4K) : AT(ADDR(.eh_frame) - __kvma_beg) { 
        PROVIDE(__eh_frame = .);
        KEEP(*(.eh_frame)) 
        *(.eh_frame.*) 
    }

    . = ALIGN(512);

    __etext = .;
    __kernel_size = . - (__kvma_beg + __kernel_load_adr);

    .bss ALIGN(4K) (NOLOAD) : {
        __bss_start = .;
        . += 4M;
		__kernel_stack_top = .;
        *(COMMON)
        *(.bss .bss.*)
        __bss_end = .;
    }
    . = ALIGN(4K);
    __kwm = .;
} 

