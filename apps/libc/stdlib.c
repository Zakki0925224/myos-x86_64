#include "stdlib.h"
#include "syscalls.h"
#include <stddef.h>

int abs(int i)
{
    return i < 0 ? -i : i;
}

void *malloc(size_t len)
{
    return sys_sbrk(len);
}

int atoi(const char *str)
{
    return -1;
}

double atof(const char *__nptr)
{
    return -1.0;
}

void free(void *ptr)
{
}

void *calloc(size_t num_elems, size_t size)
{
    return NULL;
}

void *realloc(void *ptr, size_t size)
{
    return NULL;
}

int system(const char *command)
{
    return -1;
}

int remove(const char *__filename)
{
    return -1;
}

int rename(const char *__old, const char *__new)
{
    return -1;
}
