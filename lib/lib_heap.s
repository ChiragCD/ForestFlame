section .text

extern no_more_mem, err_bad_access, create_heap, failed_setup

global alloc, def_fill, def_deref, heap_setup

heap_setup:
    mov rax, rsi
    jz failed_setup
    mov [rsp - 16], rdi
    mov [rsp - 24], rsi
    sub rsp, 24
    call create_heap
    add rsp, 24
    mov rdi, [rsp - 16]
    mov rsi, [rsp - 24]
    sub rsi, 16
    mov [rax + 8], rsi
    mov r15, rax
ret

alloc:
    cmp rdi, [r15 + 8]
    jg no_more_mem
    mov rax, r15
    add rax, 16
    add rax, [r15 + 8]
    sub rax, rdi
    sub [r15 + 8], rdi
    add rax, 5 ; tag as a pointer
ret

def_fill:
    mov rax, rdi
    mov rbx, rax
    and rbx, 5
    cmp rbx, 5
    jne err_bad_access
    sub rax, 5
    mov [rax], rsi
    mov rax, rsi
ret

def_deref:
    mov rax, rdi
    mov rbx, rax
    and rbx, 5
    cmp rbx, 5
    jne err_bad_access
    sub rax, 5
    mov rax, [rax]
ret