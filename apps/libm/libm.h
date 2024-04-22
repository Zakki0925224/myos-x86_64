#include <stdint.h>

// syscalls.c
// syscall numbers
#define SN_READ 0
#define SN_WRITE 1
#define SN_OPEN 2
#define SN_CLOSE 3
#define SN_EXIT 4
#define SN_SBRK 5

// defined file descriptor numbers
#define FDN_STDIN 0
#define FDN_STDOUT 1
#define FDN_STDERR 2

extern int64_t sys_write(uint16_t fd, const char *str, int len);
extern void sys_exit(uint64_t status);
extern void *sys_sbrk(uint64_t len);

// string.h
extern int strcmp(const char *s1, const char *s2);
extern int strlen(const char *str);

// printf.c
extern int printf(const char *str);
