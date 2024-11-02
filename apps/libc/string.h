#ifndef _STRING_H
#define _STRING_H

#include <stddef.h>

extern int strcmp(const char *s1, const char *s2);
extern size_t strlen(const char *str);
extern int split(char *str, const char regex, char **buf, size_t buflen);
extern char *concatenate(const char *strs[], int len, const char *delimiter);
extern void replace(char *src, const char target, const char replace);
extern int is_ascii(const char c);

#endif
