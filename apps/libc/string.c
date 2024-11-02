#include "stdio.h"
#include "string.h"

int strcmp(const char *s1, const char *s2)
{
    while (*s1 != '\0' && *s2 != '\0' && *s1 == *s2)
    {
        s1++;
        s2++;
    }
    return *s1 - *s2;
}

size_t strlen(const char *str)
{
    size_t res = 0;
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

char *concatenate(const char *strs[], int len, const char *delimiter)
{
    int total_len = 0;
    int delimiter_len = strlen(delimiter);
    int i, j, k = 0;

    for (i = 0; i < len; i++)
    {
        total_len += strlen(strs[i]);
        if (i < len - 1)
        {
            total_len += delimiter_len;
        }
    }

    char *str = (char *)malloc(total_len + 1);
    if (str == NULL)
    {
        return NULL;
    }

    for (i = 0; i < len; i++)
    {
        for (j = 0; j < strlen(strs[i]); j++)
        {
            str[k++] = strs[i][j];
        }

        if (i < len - 1)
        {
            for (j = 0; j < delimiter_len; j++)
            {
                str[k++] = delimiter[j];
            }
        }
    }

    str[k] = '\0';
    return str;
}

void replace(char *src, const char target, const char replace)
{
    int i = 0;
    int str_len = strlen(src);

    while (i < str_len)
    {
        if (src[i] == target)
        {
            src[i] = replace;
        }

        i++;
    }
}

int is_ascii(const char c)
{
    return c >= 0 && c <= 127;
}
