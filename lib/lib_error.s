section .text

extern snek_error

global raise_error, failed_setup, no_more_mem, type_error, err_bad_access, expect_bool, expect_numeric
global overflow

raise_error:
mov rsp, [r15]
jmp snek_error

failed_setup:
mov rdi, 1
call snek_error     ; Do not call raise error, r15 has not yet been created

no_more_mem:
mov rdi, 2
jmp raise_error

type_error:
mov rdi, 3
jmp raise_error

err_bad_access:
mov rdi, 4
jmp raise_error

expect_bool:
mov rdi, 5
jmp raise_error

expect_numeric:
mov rdi, 6
jmp raise_error

overflow:
mov rdi, 7
jmp raise_error