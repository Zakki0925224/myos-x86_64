#include "../libm/libm.h"

int strcmp(const char *s1, const char *s2)
{
    while (*s1 != '\0' && *s2 != '\0' && *s1 == *s2)
    {
        s1++;
        s2++;
    }
    return *s1 - *s2;
}

int strlen(const char *str)
{
    int res = 0;
    while (*str++)
    {
        res++;
    }
    return res;
}

void _start(int argc, char *argv[])
{
    if (strcmp(argv[1], argv[2]) != 0)
    {
        sys_exit(1);
    }

    sys_exit((uint64_t)strlen(argv[1]));
}
