.code64

.global set_segments
# fn set_segments(cs: u16, ds: u16, ss: u16, es: u16, gs: u16, fs: u16);
set_segments:
    mov rax, offset reload_cs
    push rdi
    push rax
    retfq
reload_cs:
    mov ds, si 
    mov ss, dx
    mov es, cx
    mov rax, r8
    mov gs, ax
    mov rax, r9 
    mov fs, ax
    ret