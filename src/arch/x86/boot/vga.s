.section .boot.text
.code32

vga_pos: .long 0xB8000

# Prints whatever is pointed to by the esi register. 
.global vga_print
vga_print:
    mov edi, [vga_pos]
.Lprint:
    lodsb
    or al, al
    jz .Lend
    mov ah, 3
    stosw
    jmp .Lprint 
.Lend:
    mov [vga_pos], edi
    ret

# Prints whatever is pointed to by the esi register, and then appends a newline.
.global vga_println
vga_println:
    call vga_print
    mov ebx, [vga_pos]
    mov eax, ebx
    sub eax, 0xB8000
    xor edx, edx
    mov ecx, 160
    div ecx
    mov eax, ebx
    sub ecx, edx
    add eax, ecx
    mov [vga_pos], eax
    ret

# Clears the display for further drawing.
.global vga_clear
vga_clear:
    mov eax, 80
    xor edx, edx
    mov ecx, 25
    mul ecx
    mov ecx, eax
    mov edi, 0xB8000
    mov ax, 0
    rep stosw
    ret