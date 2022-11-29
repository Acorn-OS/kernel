.global walk_stack
walk_stack:
    # Create new stack frame.
    push rbp
    mov rbp, rsp

    # Previous stack frame goes into RSI, and max array length goes into RCX.
    mov rcx, rsi
    mov rsi, [rsp]

    # The amount of stack frames found.
    xor rdx, rdx

0:
    # See if current stack frame is NULL (end of stack-trace).
    test rsi, rsi 
    jz 1f
    
    # Assigned iterated stack frame's return address to RAX, and its RBP into RSI.

    mov rax, [rsi + 8]
    mov rsi, [rsi]
    
    # Store the value of the stack frame's IP the return array
    stosq

    # Increase the found stack frame counter.
    # EDX is faster than RDX.
    inc edx

    loop 0b
1:
    # Move return value into RAX per calling convention.
    mov rax, rdx
    # Restore stack frame.
    leave 
    ret 
