#ifndef _STDIO_H
#define _STDIO_H

#include <stdarg.h>
#include <stdint.h>
#include <stddef.h>
#include "temp.h"

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
extern int mkdir(const char *__path, __mode_t __mode);

#endif
