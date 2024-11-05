#include "stdio.h"
#include "syscalls.h"
#include <stddef.h>

void exit(int status)
{
    sys_exit((uint64_t)status);
}

void *memset(void *s, int c, size_t n)
{
    return NULL;
}

int fprintf(FILE *stream, const char *fmt, ...)
{
    return -1;
}

int snprintf(char *buff, size_t size, const char *format, ...)
{
    return -1;
}

int strcasecmp(const char *s1, const char *s2)
{
    return -1;
}

char *strdup(const char *s)
{
    return NULL;
}

char *strrchr(const char *s, int c)
{
    return NULL;
}

void *memmove(void *dest, const void *src, size_t len)
{
    return NULL;
}

int strncasecmp(const char *s1, const char *s2, size_t n)
{
    return -1;
}

FILE *fopen(
    const char *filename,
    const char *mode)
{
    return NULL;
}

int fclose(FILE *stream)
{
    return -1;
}

long int ftell(FILE *__stream)
{
    return -1;
}

int fflush(FILE *__stream)
{
    return -1;
}

int puts(const char *c)
{
    return -1;
}

int putchar(int c)
{
    return -1;
}

char *strchr(const char *s, int c)
{
    return NULL;
}

int vfprintf(FILE *stream, const char *fmt, va_list ap)
{
    return -1;
}

int sscanf(const char *buf, const char *fmt, ...)
{
    return -1;
}

size_t fread(void *ptr, size_t size, size_t count, FILE *stream)
{
    return -1;
}

int fseek(FILE *__stream, long int __off, int __whence)
{
    return -1;
}

size_t fwrite(const void *buffer, size_t size, size_t count, FILE *stream)
{
    return -1;
}

char *strstr(const char *s1, const char *s2)
{
    return NULL;
}

int strncmp(const char *s1, const char *s2, size_t n)
{
    return -1;
}

char *strncpy(char *dst, const char *src, size_t n)
{
    return NULL;
}

int vsnprintf(char *buffer, size_t bufsize, const char *format, va_list arg)
{
    return -1;
}
