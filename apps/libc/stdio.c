#include "stdio.h"
#include "string.h"
#include "syscalls.h"
#include <stddef.h>

void exit(int status)
{
    sys_exit((uint64_t)status);
}

int fprintf(FILE *stream, const char *fmt, ...)
{
    printf("[DEBUG]fprintf called\n");
    return -1;
}

FILE *fopen(
    const char *filename,
    const char *mode)
{
    printf("[DEBUG]fopen called\n");
    return NULL;
}

int fclose(FILE *stream)
{
    printf("[DEBUG]fclose called\n");
    return -1;
}

long int ftell(FILE *__stream)
{
    printf("[DEBUG]ftell called\n");
    return -1;
}

int fflush(FILE *__stream)
{
    printf("[DEBUG]fflush called\n");
    return -1;
}

int puts(const char *c)
{
    int64_t ret = sys_write(FDN_STDOUT, c, strlen(c));

    if (ret == -1)
        return -1;

    return 0;
}

int putchar(int c)
{
    return printf("%c", c);
}

int vfprintf(FILE *stream, const char *fmt, va_list ap)
{
    printf("[DEBUG]vfprintf called\n");
    return -1;
}

int sscanf(const char *buf, const char *fmt, ...)
{
    printf("[DEBUG]sscanf called\n");
    return -1;
}

size_t fread(void *ptr, size_t size, size_t count, FILE *stream)
{
    printf("[DEBUG]fread called\n");
    return -1;
}

int fseek(FILE *__stream, long int __off, int __whence)
{
    printf("[DEBUG]fseek called\n");
    return -1;
}

size_t fwrite(const void *buffer, size_t size, size_t count, FILE *stream)
{
    printf("[DEBUG]fwrite called\n");
    return -1;
}
