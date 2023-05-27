%define MMAP 9
%define MUNMAP 11
%define NULLPTR 0
%define PROT_READ 0x1
%define PROT_WRITE 0x2
%define MAP_PRIVATE 0x02
%define MAP_ANONYMOUS 0x20

section .text

extern raise_error

global heap_alloc
global heap_free
global setup_base

mmap:
    mov [rsp - 16], rdi
    mov [rsp - 24], rsi
    mov [rsp - 32], rdx
    mov [rsp - 40], r10
    mov [rsp - 48], r8
    mov [rsp - 56], r9
    mov rax, MMAP
    mov rdi, NULLPTR
    mov rsi, 4096
    mov rdx, PROT_READ | PROT_WRITE
    mov r10, MAP_PRIVATE | MAP_ANONYMOUS
    mov r8, -1
    mov r9, 0
    sub rsp, 72
    syscall
    add rsp, 72
    mov rdi, [rsp - 16]
    mov rsi, [rsp - 24]
    mov rdx, [rsp - 32]
    mov r10, [rsp - 40]
    mov r8, [rsp - 48]
    mov r9, [rsp - 56]
ret

munmap:
    mov [rsp - 16], rdi
    mov [rsp - 24], rsi
    mov rax, MUNMAP
    mov rsi, 4096
    sub rsp, 40
    syscall
    add rsp, 40
    cmp rax, 0
    mov rdi, rax
    jl raise_error
    mov rdi, [rsp - 16]
    mov rsi, [rsp - 24]
ret

heap_alloc:
    call mmap
ret

heap_free:
    call munmap
ret

setup_base:
    call mmap       ; error, if any, not handled
    mov [rax], r15
    mov r15, rax
ret