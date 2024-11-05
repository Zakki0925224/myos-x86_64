#ifndef _STDIO_H
#define _STDIO_H

#include <stdarg.h>
#include <stdint.h>
#include <stddef.h>

#define SEEK_SET 0
#define SEEK_END 2

typedef struct _FILE
{
} FILE;

// extern FILE *stdin;
// extern FILE *stdout;
// extern FILE *stderr;

// #define stdin stdin
// #define stdout stdout
// #define stderr stderr

// printf.c
extern int printf(const char *fmt, ...);

extern void exit(int status);
extern int fprintf(FILE *stream, const char *fmt, ...);
extern int snprintf(char *buff, size_t size, const char *format, ...);
extern FILE *fopen(const char *filename, const char *mode);
extern int fclose(FILE *stream);
extern long int ftell(FILE *__stream);
extern int fflush(FILE *__stream);
extern int puts(const char *c);
extern int putchar(int c);
extern int vfprintf(FILE *stream, const char *fmt, va_list ap);
extern int sscanf(const char *buf, const char *fmt, ...);
extern size_t fread(void *ptr, size_t size, size_t count, FILE *stream);
extern int fseek(FILE *__stream, long int __off, int __whence);
extern size_t fwrite(const void *buffer, size_t size, size_t count, FILE *stream);
extern int vsnprintf(char *buffer, size_t bufsize, const char *format, va_list arg);

#endif
