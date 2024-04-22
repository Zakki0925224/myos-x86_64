# libm

Standard C library for myos

## Syscall tables

| SN  | name  | arg1(%rdi) | arg2(%rsi)           | arg3(%rdx)       | arg4(%r10) | arg5(%r8) | arg6(%r9) | ret(%rax) |
| --- | ----- | ---------- | -------------------- | ---------------- | ---------- | --------- | --------- | --------- |
| 0   | read  | 0x0        |                      |                  |            |           |           |           |
| 1   | write | 0x1        | uint16_t fd          | const char \*str | int len    | -         | -         | int64_t   |
| 2   | open  | 0x2        |                      |                  |            |           |           |           |
| 3   | close | 0x3        |                      |                  |            |           |           |           |
| 4   | exit  | 0x4        | uint64_t status      | -                | -          | -         | -         | void      |
| 5   | sbrk  | 0x5        | uint64_t len         | -                | -          | -         | -         | void\*    |
| 6   | uname | 0x6        | struct utsname \*buf | -                | -          | -         | -         | int64_t   |
| 7   | break | 0x7        | -                    | -                | -          | -         | -         | -         |
