BITS 64

global draw_board
global init_board
global check_win
global check_draw

; 0xE2, 0x94, 0x8C = ┌
; 0xE2, 0x94, 0x90 = ┐
; 0xE2, 0x94, 0x94 = └
; 0xE2, 0x94, 0x98 = ┘
; 0xE2, 0x94, 0x80 = ─
; 0xE2, 0x94, 0x82 = │
; 0xE2, 0x94, 0xAC = ┬
; 0xE2, 0x94, 0xB4 = ┴
; 0xE2, 0x94, 0xA4 = ┤
; 0xE2, 0x94, 0x9C = ├
; 0xE2, 0x94, 0xBC = ┼

section .data
                                 ; ┌─┬─┐
    board_top db 0xE2, 0x94, 0x8C, 0xE2, 0x94, 0x80, 0xE2, 0x94, 0xAC, 0xE2, 0x94, 0x80, 0xE2, 0x94, 0xAC, 0xE2, 0x94, 0x80, 0xE2, 0x94, 0x90, 10
    board_top_len equ $ - board_top
                                 ; ├─┼─┤
    board_middle db 0xE2, 0x94, 0x9C, 0xE2, 0x94, 0x80, 0xE2, 0x94, 0xBC, 0xE2, 0x94, 0x80, 0xE2, 0x94, 0xBC, 0xE2, 0x94, 0x80, 0xE2, 0x94, 0xA4, 10
    board_middle_len equ $ - board_middle
                                 ; └─┴─┘
    board_bottom db 0xE2, 0x94, 0x94, 0xE2, 0x94, 0x80, 0xE2, 0x94, 0xB4, 0xE2, 0x94, 0x80, 0xE2, 0x94, 0xB4, 0xE2, 0x94, 0x80, 0xE2, 0x94, 0x98, 10
    board_bottom_len equ $ - board_bottom
    board_vertical db 0xE2, 0x94, 0x82
    board_vertical_len equ $ - board_vertical
    row_len equ 16 

section .bss
    row resb 16

section .text

; Draw a single row of the game board
; Parameters:
;   rdi - pointer to the three-element row of the board
; Returns:
;   None
draw_board_row:
    mov r9, rdi
    xor r8, r8
copy_vertical_lines_loop:
    lea rdi, [row+r8*4]
    mov rsi, board_vertical
    mov rcx, board_vertical_len
    rep movsb
    inc r8
    cmp r8, 4
    jl copy_vertical_lines_loop
                                 ; Copy board data into row, spaced between the vertical bars
    mov al, [r9]
    mov byte [row + 3], al
    mov al, [r9+1]
    mov byte [row + 7], al
    mov al, [r9+2]
    mov byte [row + 11], al
    mov byte [row + 15], 0x0A    ; Newline
    mov rax, 1                   ; syscall: write
    mov rdi, 1                   ; fd = stdout
    mov rsi, row
    mov rdx, row_len
    syscall
    ret

; Draw the game board
; Parameters:
;   rdi - pointer to the board state
; Returns:
;   None
draw_board:
    mov r8, rdi
                                 ; Draw the top row
    mov rax, 1                   ; syscall: write
    mov rdi, 1                   ; fd = stdout
    mov rsi, board_top
    mov rdx, board_top_len
    syscall
                                 ; Draw the top dynamic row
    mov rdi, r8
    push r8
    call draw_board_row
    pop r8
                                 ; Draw the top-middle static row
    mov rax, 1                   ; syscall: write
    mov rdi, 1                   ; fd = stdout
    mov rsi, board_middle
    mov rdx, board_middle_len
    syscall
                                 ; Draw the middle dynamic row
    mov rdi, r8
    add rdi, 3
    push r8
    call draw_board_row
    pop r8
                                 ; Draw the bottom-middle static row
    mov rax, 1                   ; syscall: write
    mov rdi, 1                   ; fd = stdout
    mov rsi, board_middle
    mov rdx, board_middle_len
    syscall
                                 ; Draw bottom dynamic row
    mov rdi, r8
    add rdi, 6
    push r8
    call draw_board_row
    pop r8
                                 ; Draw the bottom static row
    mov rax, 1                   ; syscall: write
    mov rdi, 1                   ; fd = stdout
    mov rsi, board_bottom
    mov rdx, board_bottom_len
    syscall
    ret

; Initialise the characters of the 9-element board array
; Parameters:
;   rdi - pointer to the board state
; Returns:
;   None
init_board:
    xor rcx, rcx
init_board_loop:
    mov byte [rdi+rcx], 0x20
    inc rcx
    cmp rcx, 9
    jl init_board_loop
    ret

; Check for win line
; Parameters:
;   rdi - pointer to the board state
; Returns:
;   rax - 0x00 if no win, and the character that won otherwise ('X' or 'O')
check_win:
    push rbx
    mov rbx, rdi
    call check_horizontal_win
    cmp rax, 0x00
    jne check_win_end
    mov rdi, rbx
    call check_vertical_win
    cmp rax, 0x00
    jne check_win_end
    mov rdi, rbx
    call check_diagonal_win
check_win_end:
    pop rbx
    ret

; Check for draw
; Parameters:
;   rdi - pointer to the board state
; Returns:
;   rax - 0x00 if no draw, 0x01 if there is a draw
check_draw:
    xor rcx, rcx
check_draw_loop:
    cmp rcx, 9
    je check_draw_end
    inc rcx
    cmp byte [rdi+rcx-1], 0x20
    jne check_draw_loop
    mov rax, 0x00
    ret
check_draw_end:
    mov rax, 0x01
    ret

; Check for horizontal win line
; Parameters:
;   rdi - pointer to the board state
; Returns:
;   rax - 0x00 if no win, and the character that won otherwise ('X' or 'O')
check_horizontal_win:
    xor rcx, rcx
    mov r9, 3                    ; Multiplier for address computation
check_horizontal_win_loop:
    cmp rcx, 3
    je check_horizontal_no_win
    mov rax, rcx                 ; Compute address, rdi+rcx*3
    inc rcx
    mul r9                       ; Multiple rax by r9
    add rax, rdi
    mov r8b, [rax]
    cmp r8b, 0x20                ; Is first char a space?
    je check_horizontal_win_loop
    cmp r8b, [rax+1]             ; Are the first and second chars equal?
    jne check_horizontal_win_loop
    cmp r8b, [rax+2]             ; Are the first and third chars equal?
    jne check_horizontal_win_loop
    movzx rax, r8b               ; Zero-extend to fill register
    ret
check_horizontal_no_win:
    mov rax, 0x00
    ret

; Check for vertical win line
; Parameters:
;   rdi - pointer to the board state
; Returns:
;   rax - 0x00 if no win, and the character that won otherwise ('X' or 'O')
check_vertical_win:
    xor rcx, rcx
check_vertical_win_loop:
    cmp rcx, 3
    je check_vertical_no_win
    inc rcx
    mov r8b, [rdi+rcx-1]
    cmp r8b, 0x20                ; Is first char a space?
    je check_vertical_win_loop
    cmp r8b, [rdi+rcx+2]         ; Are the first and second chars equal?
    jne check_vertical_win_loop
    cmp r8b, [rdi+rcx+5]         ; Are the first and third chars equal?
    jne check_vertical_win_loop
    movzx rax, r8b               ; Zero-extend to fill register
    ret
check_vertical_no_win:
    mov rax, 0x00
    ret

; Check for vertical win line
; Parameters:
;   rdi - pointer to the board state
; Returns:
;   rax - 0x00 if no win, and the character that won otherwise ('X' or 'O')
check_diagonal_win:              ; Check top-left to bottom-right diagonal
    mov r8b, [rdi]
    cmp r8b, 0x20                ; Is first char a space?
    je check_diagonal_win_tr_bl
    cmp r8b, [rdi+4]             ; Are the first and second chars equal?
    jne check_diagonal_win_tr_bl
    cmp r8b, [rdi+8]             ; Are the first and third chars equal?
    jne check_diagonal_win_tr_bl
    movzx rax, r8b               ; Zero-extend to fill register
    ret
check_diagonal_win_tr_bl:        ; Check top-right to bottom-left diagonal
    mov r8b, [rdi+2]
    cmp r8b, 0x20                ; Is first char a space?
    je check_diagonal_no_win
    cmp r8b, [rdi+4]             ; Are the first and second chars equal?
    jne check_diagonal_no_win
    cmp r8b, [rdi+6]             ; Are the first and third chars equal?
    jne check_diagonal_no_win
    movzx rax, r8b               ; Zero-extend to fill register
    ret
check_diagonal_no_win:
    mov rax, 0x00
    ret
