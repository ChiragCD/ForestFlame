section .text

extern snek_print

global def_print

def_print:
    mov [rsp - 16], rsi
    mov [rsp - 24], r8
    mov [rsp - 32], r9
    mov [rsp - 40], rdx
    mov [rsp - 48], rcx
    sub rsp, 56
    call snek_print
    add rsp, 56
    mov rsi, [rsp - 16]
    mov r8, [rsp - 24]
    mov r9, [rsp - 32]
    mov rdx, [rsp - 40]
    mov rcx, [rsp - 48]
ret