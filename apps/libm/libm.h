#include <stdint.h>
#include "utsname.h"

// syscalls.c
// syscall numbers
#define SN_READ 0
#define SN_WRITE 1
#define SN_OPEN 2
#define SN_CLOSE 3
#define SN_EXIT 4
#define SN_SBRK 5
#define SN_UNAME 6
#define SN_BREAK 7

// defined file descriptor numbers
#define FDN_STDIN 0
#define FDN_STDOUT 1
#define FDN_STDERR 2

extern int64_t sys_read(int64_t fd, void *buf, int buf_len);
extern int64_t sys_write(int64_t fd, const char *str, int len);
extern int64_t sys_open(const char *filepath);
extern int64_t sys_close(int64_t fd);
extern void sys_exit(uint64_t status);
extern void *sys_sbrk(uint64_t len);
extern int64_t sys_uname(struct utsname *buf);
extern void sys_break(void);

// string.h
extern int strcmp(const char *s1, const char *s2);
extern int strlen(const char *str);

// printf.c
extern int printf(const char *str);
