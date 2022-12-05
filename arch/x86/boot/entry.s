.section .boot.entry 
.code16

.global _start
_start:
    ret

# Reserve space for partitions.
.org 0x1B8

# MBR magic number.
.org 510
.byte 0x55, 0xAA
