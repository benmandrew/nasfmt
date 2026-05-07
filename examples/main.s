BITS 64

section .data
    prompt db "Enter a number (1-9):", 10, 0
    prompt_len equ $ - prompt
    invalid db "Invalid input.", 10, 0
    invalid_len equ $ - invalid
    newline db 10
    reset_cursor_ansi db `\033[8A\033[14D`
    reset_cursor_ansi_len equ $ - reset_cursor_ansi
    winner db "The winner is: ", 0
    winner_len equ $ - winner
    draw db "It was a draw!", 10, 0
    draw_len equ $ - draw
    no_stdin db "Read syscall returned 0. is STDIN connected?", 10, 0
    no_stdin_len equ $ - no_stdin

section .bss
    input_char resb 1
    winner_char resb 1
    current_player_char resb 1
    board resb 9

section .text
    global _start

extern set_raw_mode
extern draw_board
extern init_board
extern check_win
extern check_draw

_start:
    call set_raw_mode
    mov rdi, board
    call init_board
    mov rdi, board
    call draw_board
    mov byte [current_player_char], 'X'
game_loop:
    mov rdi, board
    call get_input
    mov dl, [current_player_char]
    mov [board+rax], dl
    call swap_player
    mov [current_player_char], al
                                    ; call reset_cursor
    mov rdi, board
    call draw_board
    mov rdi, board
    call check_win                  ; Check if there is a win
    mov rdi, rax
    cmp rax, 0x00
    jne game_loop_win
    mov rdi, board
    call check_draw                 ; Check if there is a draw
    cmp rax, 0x00
    jne game_loop_draw
    jmp game_loop
game_loop_win:
    call print_win
    jmp exit
game_loop_draw:
    call print_draw
    jmp exit

; Swap which player's turn it is
; Parameters:
;   dl - Current player's character
; Returns:
;   al - Next player's character
swap_player:
    cmp dl, 'X'
    je swap_player_end
    mov al, 'X'
    ret
swap_player_end:
    mov al, 'O'
    ret

; Get and validate the user's square choice
; Parameters:
;   rdi - Pointer to the board representation
; Returns:
;   rax - User's choice of index
get_input:
    push rbx
    mov rbx, rdi
    push r12
    mov r12, 0
                                    ; Prompt user
    mov rax, 1                      ; syscall: write
    mov rdi, 1                      ; fd = stdout
    mov rsi, prompt
    mov rdx, prompt_len
    syscall
get_input_start:
                                    ; Get user input
    mov rax, 0                      ; syscall: read
    mov rdi, 0                      ; fd = stdin
    mov rsi, input_char
    mov rdx, 1
    syscall
    cmp rax, 0
    jle no_input_connected
    movzx rax, byte [input_char]    ; Convert ASCII to integer
    sub rax, '0'
    cmp rax, 1                      ; Check lower bound
    jl invalid_input
    cmp rax, 9                      ; Check upper bound
    jg invalid_input
    dec rax                         ; Decrement user input to get index
    cmp byte [rbx+rax], 0x20        ; Check if chosen square is empty
    jne invalid_input
    pop r12
    pop rbx
    ret
invalid_input:
    cmp r12, 0                      ; Only notify of invalid input the first time
    jne get_input_start
    mov r12, 1
    mov rax, 1                      ; syscall: write
    mov rdi, 1                      ; fd = stdout
    mov rsi, invalid
    mov rdx, invalid_len
    syscall
    jmp get_input_start

; Reset cursor to where we started in the terminal
; Parameters:
;   None
; Returns:
;   None
reset_cursor:
    mov rax, 1                      ; syscall: write
    mov rdi, 1                      ; fd = stdout
    mov rsi, reset_cursor_ansi
    mov rdx, reset_cursor_ansi_len
    syscall
    ret

; Print whoever won
; Parameters:
;   rdi - character for the winner
; Returns:
;   None
print_win:
    mov [winner_char], dil          ; lowest 8 bits of rdi
    mov rax, 1                      ; syscall: write
    mov rdi, 1                      ; fd = stdout
    mov rsi, winner
    mov rdx, winner_len
    syscall
    mov rax, 1
    mov rdi, 1
    mov rsi, winner_char
    mov rdx, 1
    syscall
    mov rax, 1
    mov rdi, 1
    mov rsi, newline
    mov rdx, 1
    syscall
    ret

; Print draw text
; Parameters:
;   None
; Returns:
;   None
print_draw:
    mov rax, 1
    mov rdi, 1
    mov rsi, draw
    mov rdx, draw_len
    syscall
    ret

no_input_connected:
    mov rax, 1                      ; syscall: write
    mov rdi, 1                      ; fd = stdout
    mov rsi, no_stdin
    mov rdx, no_stdin_len
    syscall
    jmp exit


exit:
    mov rdi, 0                      ; exit code
    mov rax, 60                     ; syscall: exit
    syscall
