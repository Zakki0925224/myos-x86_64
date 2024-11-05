#include "ctype.h"

int toupper(int c)
{
    if (c >= 'a' && c <= 'z')
    {
        return (c - 'a' + 'A');
    }

    return c;
}

int isspace(int c)
{
    return ((c == ' ') || (c == '\n') || (c == '\t'));
}
