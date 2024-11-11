#ifndef _STDLIB_H
#define _STDLIB_H

#include <stddef.h>

extern int abs(int i);
extern void *malloc(size_t len);
extern int atoi(const char *str);
extern double atof(const char *__nptr);
extern void free(void *ptr);
extern void *calloc(size_t count, size_t size);
extern void *realloc(void *ptr, size_t size);
extern int system(const char *command);
extern int remove(const char *__filename);
extern int rename(const char *__old, const char *__new);

#endif
