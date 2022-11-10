ENTRY(_start)
OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)

SECTIONS {
    kernel_virt_adr = 0xFFFFFFFF80000000;
    kernel_virt_end = 0xFFFFFFFFFFFFFFFF;
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

    
    . = kernel_virt_adr + kernel_load_adr;

    .setup : AT(ADDR(.setup) - kernel_virt_adr) {
        KEEP(*(.setup))
    }

    .text ALIGN(4K) : AT(ADDR(.text) - kernel_virt_adr) {
        *(.text .text.*)
    }

    .data ALIGN(4K) : AT(ADDR(.data) - kernel_virt_adr) {
        *(.data .data.*)
    }

    .misc ALIGN(4K) : AT(ADDR(.misc) - kernel_virt_adr) {
        *(.eh_frame)
        *(.eh_frame_hdr)
    }

    .rodata ALIGN(4K) : AT(ADDR(.rodata) - kernel_virt_adr) {
        *(.rodata .rodata.*)
    }

    kernel_size = . - (kernel_virt_adr + kernel_load_adr);

    .bss ALIGN(4K) (NOLOAD) : {
        bss_start = .;
        . += 4M;
		kernel_stack_top = .;
        *(COMMON)
        *(.bss .bss.*)
        bss_end = .;
    }

    . = ALIGN(4K);

    . = . - kernel_virt_adr;

    kernel_free_mem_adr = .;
    . += 16M;
    . = ALIGN(4K);
    base_page_table = .;
    kernel_free_mem_len = .;
} 

