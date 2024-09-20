#include "libm.h"

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

int split(char *str, const char regex, char **buf, size_t buflen)
{
    int s_len = strlen(str);
    int len = 1;
    int i;

    buf[0] = str;

    for (i = 1; i < s_len; i++)
    {
        if (str[i] == regex)
        {
            str[i] = '\0';
            buf[len] = &str[i + 1];
            len++;

            if ((int)buflen <= len)
            {
                break;
            }
        }
    }

    return len;
}
