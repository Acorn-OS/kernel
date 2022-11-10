# GDT access bits.
.equ GDT_PRESENT,     (1 << 7)
.equ GDT_NOT_SYS,     (1 << 4)
.equ GDT_EXEC,        (1 << 3)
.equ GDT_DC,          (1 << 2)
.equ GDT_RW,          (1 << 1)
.equ GDT_ACCESSED,    1 

# GDT flag bits.
.equ GDT_GRAN_4K,     (1 << 7)
.equ GDT_SZ_32,       (1 << 6)
.equ GDT_LONG_MODE,   (1 << 5)