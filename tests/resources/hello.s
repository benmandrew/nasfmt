global main
extern printf

section .data
    msg     db  "Hello, world!", 0
    newline db  10

section .text
main:
    push    rbp
    mov     rbp, rsp
    lea     rdi, [rel msg]
    xor     eax, eax
    call    printf
    xor     eax, eax
    pop     rbp
    ret
