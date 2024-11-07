#include "stdio.h" // for printf

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
    printf("[DEBUG]atoi called\n");
    return -1;
}

double atof(const char *__nptr)
{
    printf("[DEBUG]atof called\n");
    return -1.0;
}

void free(void *ptr)
{
    printf("[DEBUG]free called\n");
}

void *calloc(size_t num_elems, size_t size)
{
    printf("[DEBUG]calloc called\n");
    return NULL;
}

void *realloc(void *ptr, size_t size)
{
    printf("[DEBUG]realloc called\n");
    return NULL;
}

int system(const char *command)
{
    printf("[DEBUG]system called (command: %s)\n", command);
    return -1;
}

int remove(const char *__filename)
{
    printf("[DEBUG]remove called\n");
    return -1;
}

int rename(const char *__old, const char *__new)
{
    printf("[DEBUG]rename called\n");
    return -1;
}
