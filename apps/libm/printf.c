#include <stdarg.h>
#include "libm.h"

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
    BUF[buf_i++] = c;
    return buf_i;
}

int printf(const char *fmt, ...)
{
    int ret = 0;
    va_list ap;
    va_start(ap, fmt);

    int str_len = strlen(fmt);
    int str_i = 0;
    int buf_i = 0;

    if (str_len <= 0)
    {
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

        char c = fmt[str_i++];

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

        char nc = fmt[str_i++];
        switch (nc)
        {
        case 'd':
        {
            int num = va_arg(ap, int);
            int tmp;
            int num_digit = 0;

            if (num == 0)
            {
                buf_i = push_buf_and_write(buf_i, '0');
                break;
            }
            else if (num < 0)
            {
                buf_i = push_buf_and_write(buf_i, '-');
                num = -num;
            }

            tmp = num;
            while (tmp > 0)
            {
                tmp /= 10;
                num_digit++;
            }

            if (num_digit >= BUF_SIZE)
            {
                ret = -1;
                break;
            }

            if (buf_i + num_digit >= BUF_SIZE)
            {
                if (write_buf(buf_i) == -1)
                {
                    ret = -1;
                    break;
                }
            }

            for (int i = num_digit - 1; i >= 0; i--)
            {
                int digit = num % 10;
                num /= 10;
                push_buf_and_write(buf_i + i, '0' + digit);
            }

            buf_i += num_digit;
            break;
        }

        case 'c':
        {
            char cfv = va_arg(ap, int);
            buf_i = push_buf_and_write(buf_i, cfv);
            break;
        }
        case 's':
        {
            const char *s = va_arg(ap, char *);
            int s_len = strlen(s);
            for (int i = 0; i < s_len; i++)
            {
                buf_i = push_buf_and_write(buf_i, s[i]);
            }
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
