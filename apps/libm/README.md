# libm

Standard C library for myos

## Syscall tables

| SN  | name  | description                            | arg1(%rdi) | arg2(%rsi)            | arg3(%rdx)       | arg4(%r10)  | arg5(%r8) | arg6(%r9) | ret(%rax)                                      |
| --- | ----- | -------------------------------------- | ---------- | --------------------- | ---------------- | ----------- | --------- | --------- | ---------------------------------------------- |
| 0   | read  | Read file                              | 0x0        | uint64_t fd           | void \*buf       | int buf_len |           |           | int64_t (success: 0, error: -1)                |
| 1   | write | Write file                             | 0x1        | uint64_t fd           | const char \*str | int len     | -         | -         | int64_t (success: 0, error: -1)                |
| 2   | open  | Open file                              | 0x2        | const char \*filepath |                  |             |           |           | int64_t (success: uint64_t fd, error: -1)      |
| 3   | close | Close file                             | 0x3        | uint64_t fd           |                  |             |           |           | int64_t (success: 0, error: -1)                |
| 4   | exit  | Exit app with status (noreturn)        | 0x4        | uint64_t status       | -                | -           | -         | -         | void                                           |
| 5   | sbrk  | Allocate memory (4KB align)            | 0x5        | uint64_t len          | -                | -           | -         | -         | void\* (success: pointer, error: null pointer) |
| 6   | uname | Get system information                 | 0x6        | struct utsname \*buf  | -                | -           | -         | -         | int64_t (success: 0, error: -1)                |
| 7   | break | Trap at current instruction (noreturn) | 0x7        | -                     | -                | -           | -         | -         | void                                           |
