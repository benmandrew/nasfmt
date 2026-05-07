BITS 64

global set_raw_mode

section .bss
    termios_raw resb 32            ; struct termios is 32 bytes on x86-64

section .text

; Terminal raw mode for immediate user input without waiting for enter
set_raw_mode:
                                   ; TCGETS = 0x5401, TCSETS = 0x5402
    mov rax, 16                    ; syscall: ioctl
    mov rdi, 0                     ; fd = stdin
    mov rsi, 0x5401                ; TCGETS
    mov rdx, termios_raw
    syscall

                                   ; Modify termios_raw: unset ICANON (0x0002) and ECHO (0x0008) in c_lflag
                                   ; c_lflag is at offset 12 in termios struct
    mov rbx, termios_raw
    add rbx, 12
    mov eax, [rbx]
    and eax, 0xFFFFFFFFFFFFFFF5    ; clear ICANON (bit 1) and ECHO (bit 3)
    mov [rbx], eax

    mov rax, 16                    ; syscall: ioctl
    mov rdi, 0                     ; fd = stdin
    mov rsi, 0x5402                ; TCSETS
    mov rdx, termios_raw
    syscall
    ret
