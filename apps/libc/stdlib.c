#include "stdio.h" // for printf

#include "stdlib.h"
#include "string.h"
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

void *calloc(size_t count, size_t size)
{
    // printf("[DEBUG]calloc called\n");
    void *ptr = malloc(count * size);
    if (ptr == NULL)
        return NULL;

    memset(ptr, 0, count * size);
    return ptr;
}

void *realloc(void *ptr, size_t size)
{
    // printf("[DEBUG]realloc called\n");
    void *new_ptr = malloc(size);
    if (new_ptr == NULL)
        return NULL;

    memcpy(new_ptr, ptr, size);
    free(ptr);
    return new_ptr;
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
