#include <stdarg.h>
#include <stdio.h>
#include "syscalls.h"
#include "string.h"

#define BUF_SIZE 100
static char BUF[BUF_SIZE];

int write_buf(int buf_i)
{
    int64_t ret = sys_write(FDN_STDOUT, BUF, buf_i);

    if (ret == -1)
    {
        return -1;
    }
    return 0;
}

int push_buf_and_write(int buf_i, char c)
{
    if (buf_i >= BUF_SIZE)
    {
        // write
        write_buf(buf_i);
        buf_i = 0;
    }
    BUF[buf_i++] = is_ascii(c) ? c : '?';
    return buf_i;
}

int printf(const char *fmt, ...)
{
    int ret = 0;
    va_list ap;
    va_start(ap, fmt);

    int i;
    int str_len = strlen(fmt);
    int str_i = 0;
    int buf_i = 0;
    char c, nc;

    int va_num, va_num_tmp, va_num_digit, digit;
    char va_c;
    const char *va_s = NULL;
    int va_s_len;

    if (str_len <= 0)
    {
        va_end(ap);
        return 0;
    }

    for (;;)
    {
        if (str_i >= str_len)
        {
            // write
            if (write_buf(buf_i) == -1)
            {
                ret = -1;
            }
            break;
        }

        c = fmt[str_i++];

        if (c != '%')
        {
            buf_i = push_buf_and_write(buf_i, c);
            continue;
        }

        if (str_i >= str_len)
        {
            // write buf at next loop
            continue;
        }

        nc = fmt[str_i++];
        switch (nc)
        {
        case 'd':
        {
            va_num = va_arg(ap, int);
            va_num_digit = 0;

            if (va_num == 0)
            {
                buf_i = push_buf_and_write(buf_i, '0');
                break;
            }
            else if (va_num < 0)
            {
                buf_i = push_buf_and_write(buf_i, '-');
                va_num = -va_num;
            }

            va_num_tmp = va_num;
            while (va_num_tmp > 0)
            {
                va_num_tmp /= 10;
                va_num_digit++;
            }

            if (va_num_digit >= BUF_SIZE)
            {
                ret = -1;
                break;
            }

            if (buf_i + va_num_digit >= BUF_SIZE)
            {
                if (write_buf(buf_i) == -1)
                {
                    ret = -1;
                    break;
                }
            }

            for (i = va_num_digit - 1; i >= 0; i--)
            {
                digit = va_num % 10;
                va_num /= 10;
                push_buf_and_write(buf_i + i, '0' + digit);
            }

            buf_i += va_num_digit;
            break;
        }

        case 'c':
        {
            va_c = va_arg(ap, int);
            buf_i = push_buf_and_write(buf_i, va_c);
            break;
        }
        case 's':
        {
            va_s = va_arg(ap, char *);
            if (va_s == NULL)
            {
                ret = -1;
                break;
            }

            va_s_len = strlen(va_s);
            for (i = 0; i < va_s_len; i++)
            {
                buf_i = push_buf_and_write(buf_i, va_s[i]);
            }
            va_s = NULL;
            break;
        }

        case '%':
            buf_i = push_buf_and_write(buf_i, '%');
            break;

        default:
            ret = -1;
            break;
        }

        if (ret == -1)
        {
            break;
        }
    }

    va_end(ap);

    // TODO: debugging
    if (ret == -1)
    {
        printf("<PRINTF ERROR>\n");
    }

    return ret;
}
