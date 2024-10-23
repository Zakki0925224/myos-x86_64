# libm

Standard C Library for MyOS

## Syscalls

### read

Reads from a file.

### write

Writes to a file.

### open

Opens a file.

### close

Closes a file.

### exit

Exits the application with a status (noreturn).

### sbrk

Allocates memory, aligned to 4KB.

### uname

Retrieves system information.

### break

Triggers a trap at the current instruction (noreturn).

### stat

Gets file information.

### uptime

Returns the system uptime in milliseconds.

### exec

Executes an ELF file.

### getcwd

Gets the absolute path of the current working directory.

### chdir

Changes the current working directory.

### create_window

Creates a window.

### destroy_window

Destroys a window.

### getcwdenames

Retrieves a list of entry names in the current working directory, separated by null characters (\0).

## Syscall tables

| number | name               | arg1(%rdi) | arg2(%rsi)            | arg3(%rdx)        | arg4(%r10)     | arg5(%r8)      | arg6(%r9)       | ret(%rax)                                      |
| ------ | ------------------ | ---------- | --------------------- | ----------------- | -------------- | -------------- | --------------- | ---------------------------------------------- |
| 0      | sys_read           | 0x0        | int64_t fd            | void \*buf        | int buf_len    | -              | -               | int64_t (success: 0, error: -1)                |
| 1      | sys_write          | 0x1        | int64_t fd            | const char \*str  | int len        | -              | -               | int64_t (success: 0, error: -1)                |
| 2      | sys_open           | 0x2        | const char \*filepath | -                 | -              | -              | -               | int64_t (success: fd, error: -1)               |
| 3      | sys_close          | 0x3        | int64_t fd            | -                 | -              | -              | -               | int64_t (success: 0, error: -1)                |
| 4      | sys_exit           | 0x4        | uint64_t status       | -                 | -              | -              | -               | void                                           |
| 5      | sys_sbrk           | 0x5        | uint64_t len          | -                 | -              | -              | -               | void\* (success: pointer, error: null pointer) |
| 6      | sys_uname          | 0x6        | struct utsname \*buf  | -                 | -              | -              | -               | int64_t (success: 0, error: -1)                |
| 7      | sys_break          | 0x7        | -                     | -                 | -              | -              | -               | void                                           |
| 8      | sys_stat           | 0x8        | int64_t fd            | struct stat \*buf | -              | -              | -               | int64_t (success: 0, error: -1)                |
| 9      | sys_uptime         | 0x9        | -                     | -                 | -              | -              | -               | uint64_t                                       |
| 10     | sys_exec           | 0xa        | char \*args           | -                 | -              | -              | -               | int64_t (success: 0, error: -1)                |
| 11     | sys_getcwd         | 0xb        | char \*buf            | int buf_len       | -              | -              | -               | int64_t (success: 0, error: -1)                |
| 12     | sys_chdir          | 0xc        | char \*path           | -                 | -              | -              | -               | int64_t (success: 0, error: -1)                |
| 13     | sys_create_window  | 0xd        | char \*title          | uint64_t x_pos    | uint64_t y_pos | uint64_t width | uint64_t height | int64_t (success: wd, error: -1)               |
| 14     | sys_destroy_window | 0xe        | int64_t wd            | -                 | -              | -              | -               | int64_t (success: 0, error: -1)                |
| 15     | sys_getcwdenames   | 0xf        | char \*buf            | int buf_len       | -              | -              | -               | int64_t (success: 0, error: -1)                |
