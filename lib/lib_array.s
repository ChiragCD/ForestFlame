%define ARRAY_TYPE 19        ;0b 10 011

section .text

extern alloc, err_bad_access, type_error, expect_numeric

global def_array, def_index

def_array:
    test rdi, 1
    jnz expect_numeric
    shl rdi, 2          ;allocate 8*size, 1 shift already in numeric
    add rdi, 16         ;type byte, size byte
    sub rsp, 24
    call alloc
    add rsp, 24
    sub rax, 5
    mov rbx, ARRAY_TYPE
    mov [rax], rbx
    sub rdi, 16
    mov [rax + 8], rdi
    add rax, 5
ret

def_index:
    mov rax, rdi
    mov rbx, rax
    and rbx, 5
    cmp rbx, 5
    jne err_bad_access
    sub rax, 5
    mov rbx, [rax]
    cmp rbx, ARRAY_TYPE
    jne type_error
    test rsi, 1
    jnz expect_numeric
    mov rbx, [rax + 8]
    shl rsi, 2          ;compare size in bytes, 1 shift already in number
    cmp rsi, rbx
    jge err_bad_access  ;out of bounds
    cmp rsi, 0
    jl err_bad_access   ;no negative indices
    add rax, 16
    add rax, rsi
    add rax, 5
ret