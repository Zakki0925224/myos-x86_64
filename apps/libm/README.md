# libm

Standard C library for myos

## Syscalls

### read

Read file

### write

Write file

### open

Open file

### close

Close file

### exit

Exit app with status (noreturn)

### sbrk

Allocate memory (4KB align)

### uname

Get system information

### break

Trap at current instruction (noreturn)

## Syscall tables

| number | name  | arg1(%rdi) | arg2(%rsi)            | arg3(%rdx)       | arg4(%r10)  | arg5(%r8) | arg6(%r9) | ret(%rax)                                      |
| ------ | ----- | ---------- | --------------------- | ---------------- | ----------- | --------- | --------- | ---------------------------------------------- |
| 0      | read  | 0x0        | int64_t fd            | void \*buf       | int buf_len | -         | -         | int64_t (success: 0, error: -1)                |
| 1      | write | 0x1        | int64_t fd            | const char \*str | int len     | -         | -         | int64_t (success: 0, error: -1)                |
| 2      | open  | 0x2        | const char \*filepath | -                | -           | -         | -         | int64_t (success: fd, error: -1)               |
| 3      | close | 0x3        | int64_t fd            | -                | -           | -         | -         | int64_t (success: 0, error: -1)                |
| 4      | exit  | 0x4        | uint64_t status       | -                | -           | -         | -         | void                                           |
| 5      | sbrk  | 0x5        | uint64_t len          | -                | -           | -         | -         | void\* (success: pointer, error: null pointer) |
| 6      | uname | 0x6        | struct utsname \*buf  | -                | -           | -         | -         | int64_t (success: 0, error: -1)                |
| 7      | break | 0x7        | -                     | -                | -           | -         | -         | void                                           |
