#ifndef _LIBM_H
#define _LIBM_H

#include <stdarg.h>
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
extern int64_t sys_stat(int64_t fd, f_stat *buf);
extern uint64_t sys_uptime();
extern int64_t sys_exec(const char *args);
extern int64_t sys_getcwd(char *buf, int buf_len);
extern int64_t sys_chdir(const char *path);
extern int64_t sys_create_window(const char *title, uint64_t x_pos, uint64_t y_pos, uint64_t width, uint64_t height);
extern int64_t sys_destroy_window(int64_t wd);
extern int64_t sys_getcwdenames(char *buf, int buf_len);

// string.h
extern int strcmp(const char *s1, const char *s2);
extern size_t strlen(const char *str);
extern int split(char *str, const char regex, char **buf, size_t buflen);
extern char *concatenate(const char *strs[], int len, const char *delimiter);
extern void replace(char *src, const char target, const char replace);
extern int is_ascii(const char c);

// printf.c
extern int printf(const char *fmt, ...);

// malloc.c
extern void *malloc(size_t len);

// exit.c
extern void exit(int status);

// abs.c
extern int abs(int i);

// tempolary
extern void *memset(void *s, int c, size_t n);
extern int fprintf(FILE *stream, const char *fmt, ...);
extern int snprintf(char *buff, size_t size, const char *format, ...);
extern int strcasecmp(const char *s1, const char *s2);
extern char *strdup(const char *s);
extern void free(void *ptr);
extern char *strrchr(const char *s, int c);
extern void *memcpy(void *dest, const void *src, size_t len);
extern void *memmove(void *dest, const void *src, size_t len);
extern int strncasecmp(const char *s1, const char *s2, size_t n);
extern int atoi(const char *str);
extern FILE *fopen(const char *filename, const char *mode);
extern int fclose(FILE *stream);
extern int remove(const char *__filename);
extern long int ftell(FILE *__stream);
extern int rename(const char *__old, const char *__new);
extern int fflush(FILE *__stream);
extern int puts(const char *c);
extern int putchar(int c);
extern int system(const char *command);
extern char *strchr(const char *s, int c);
extern int vfprintf(FILE *stream, const char *fmt, va_list ap);
extern int sscanf(const char *buf, const char *fmt, ...);
extern double atof(const char *__nptr);
extern size_t fread(void *ptr, size_t size, size_t count, FILE *stream);
extern int fseek(FILE *__stream, long int __off, int __whence);
extern size_t fwrite(const void *buffer, size_t size, size_t count, FILE *stream);
extern char *strstr(const char *s1, const char *s2);
extern int strncmp(const char *s1, const char *s2, size_t n);
extern char *strncpy(char *dst, const char *src, size_t n);
extern int vsnprintf(char *buffer, size_t bufsize, const char *format, va_list arg);
extern void *realloc(void *ptr, size_t size);
extern void *calloc(size_t num_elems, size_t size);
extern double fabs(double x);
extern int toupper(int __c);
extern int mkdir(const char *__path, __mode_t __mode);

#endif
