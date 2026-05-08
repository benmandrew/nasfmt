section .text

load_values:
    ; Load from stack frame
    mov     rax, qword [rbp - 8]
    mov     rbx, qword [rbp - 16]
    lea     rcx, [rax + rbx * 4]
    movzx   eax, byte [rsp + rcx]
    ret

store_values:
    push    rbp
    mov     rbp, rsp
    mov     qword [rbp - 8], rdi
    mov     qword [rbp - 16], rsi
    pop     rbp
    ret
