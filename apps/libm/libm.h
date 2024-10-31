#ifndef _LIBM_H
#define _LIBM_H

#include <stdint.h>
#include <stddef.h>
#include "utsname.h"
#include "stat.h"
#include "temp.h"

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
#define SN_STAT 8
#define SN_UPTIME 9
#define SN_EXEC 10
#define SN_GETCWD 11
#define SN_CHDIR 12
#define SN_CREATE_WINDOW 13
#define SN_DESTROY_WINDOW 14
#define SN_GETCWDENAMES 15

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
extern int64_t sys_uname(utsname *buf);
extern void sys_break();
extern int64_t sys_stat(int64_t fd, stat *buf);
extern uint64_t sys_uptime();
extern int64_t sys_exec(const char *args);
extern int64_t sys_getcwd(char *buf, int buf_len);
extern int64_t sys_chdir(const char *path);
extern int64_t sys_create_window(const char *title, uint64_t x_pos, uint64_t y_pos, uint64_t width, uint64_t height);
extern int64_t sys_destroy_window(int64_t wd);
extern int64_t sys_getcwdenames(char *buf, int buf_len);

// string.h
extern int strcmp(const char *s1, const char *s2);
extern int strlen(const char *str);
extern int split(char *str, const char regex, char **buf, size_t buflen);
extern char *concatenate(const char *strs[], int len, const char *delimiter);
extern void replace(char *src, const char target, const char replace);
extern int is_ascii(const char c);

// printf.c
extern int printf(const char *fmt, ...);

// malloc.c
extern void *malloc(size_t len);

// tempolary
extern void *memset(void *s, int c, size_t n);
extern int fprintf(FILE *stream, const char *fmt, ...);
extern int snprintf(char *buff, size_t size, const char *format, ...);

#endif