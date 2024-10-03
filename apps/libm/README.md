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

### stat

Get file information

### uptime

Get system uptime in milliseconds

### exec

Execute ELF file

### getcwd

Get current working directory absolute path

### chdir

Change directory

## Syscall tables

| number | name       | arg1(%rdi) | arg2(%rsi)            | arg3(%rdx)        | arg4(%r10)  | arg5(%r8) | arg6(%r9) | ret(%rax)                                      |
| ------ | ---------- | ---------- | --------------------- | ----------------- | ----------- | --------- | --------- | ---------------------------------------------- |
| 0      | sys_read   | 0x0        | int64_t fd            | void \*buf        | int buf_len | -         | -         | int64_t (success: 0, error: -1)                |
| 1      | sys_write  | 0x1        | int64_t fd            | const char \*str  | int len     | -         | -         | int64_t (success: 0, error: -1)                |
| 2      | sys_open   | 0x2        | const char \*filepath | -                 | -           | -         | -         | int64_t (success: fd, error: -1)               |
| 3      | sys_close  | 0x3        | int64_t fd            | -                 | -           | -         | -         | int64_t (success: 0, error: -1)                |
| 4      | sys_exit   | 0x4        | uint64_t status       | -                 | -           | -         | -         | void                                           |
| 5      | sys_sbrk   | 0x5        | uint64_t len          | -                 | -           | -         | -         | void\* (success: pointer, error: null pointer) |
| 6      | sys_uname  | 0x6        | struct utsname \*buf  | -                 | -           | -         | -         | int64_t (success: 0, error: -1)                |
| 7      | sys_break  | 0x7        | -                     | -                 | -           | -         | -         | void                                           |
| 8      | sys_stat   | 0x8        | int64_t fd            | struct stat \*buf | -           | -         | -         | int64_t (success: 0, error: -1)                |
| 9      | sys_uptime | 0x9        | -                     | -                 | -           | -         | -         | uint64_t                                       |
| 10     | sys_exec   | 0xa        | char \*args           | -                 | -           | -         | -         | int64_t (success: 0, error: -1)                |
| 11     | sys_getcwd | 0xb        | char \*buf            | int buf_len       | -           | -         | -         | int64_t (success: 0, error: -1)                |
| 12     | sys_chdir  | 0xc        | char \*path           | -                 | -           | -         | -         | int64_t (success: 0, error: -1)                |
