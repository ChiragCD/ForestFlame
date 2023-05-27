%define LINK_TYPE 11        ;0b 01 011

section .text

extern alloc, type_error, err_bad_access

global def_link, def_link_from, def_link_to

def_link:
    mov [rsp - 16], rdi
    mov rdi, 24             ;extra byte for type
    sub rsp, 24
    call alloc
    add rsp, 24
    sub rax, 5
    mov rbx, LINK_TYPE
    mov [rax], rbx
    mov rdi, [rsp - 16]
    add rax, 5
ret

def_link_from:
    mov rax, rdi
    mov rbx, rax
    and rbx, 5
    cmp rbx, 5
    jne err_bad_access
    sub rax, 5
    mov rbx, [rax]
    cmp rbx, LINK_TYPE
    jne type_error
    add rax, 8
    add rax, 5              ;return mem loc
ret

def_link_to:
    mov rax, rdi
    mov rbx, rax
    and rbx, 5
    cmp rbx, 5
    jne err_bad_access
    sub rax, 5
    mov rbx, [rax]
    cmp rbx, LINK_TYPE
    jne type_error
    add rax, 16
    add rax, 5              ;return mem loc
ret