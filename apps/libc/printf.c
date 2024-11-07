#include <stdarg.h>
#include <stdio.h>
#include "syscalls.h"
#include "string.h"

#define BUF_SIZE 1000
static char BUF[BUF_SIZE];

int write_buf(char *buf, int buf_len, int write_len, char c)
{
    if (write_len >= buf_len)
        return -1;

    buf[write_len++] = c;
    return write_len;
}

int _printf(char *buf, int buf_len, const char *fmt, va_list ap)
{
    int ret = 0;
    int str_len = strlen(fmt);
    int str_i = 0;
    int buf_i = 0;

    if (str_len >= buf_len)
    {
        return -1;
    }

    for (;;)
    {
        if (str_i >= str_len)
        {
            ret = write_buf(buf, buf_len, buf_i, '\0');
            break;
        }

        char c = fmt[str_i++];

        if (c != '%')
        {
            buf_i = write_buf(buf, buf_len, buf_i, c);
            if (buf_i == -1)
            {
                ret = -1;
                break;
            }
            continue;
        }

        if (str_i >= str_len)
            continue;

        char nc = fmt[str_i++];
        switch (nc)
        {
        case 'd':
        {
            int va_num = va_arg(ap, int);
            if (va_num == 0)
            {
                buf_i = write_buf(buf, buf_len, buf_i, '0');
                break;
            }
            else if (va_num < 0)
            {
                buf_i = write_buf(buf, buf_len, buf_i, '-');
                va_num = -va_num;
            }

            char num_str[20];
            int num_len = 0;
            while (va_num > 0)
            {
                num_str[num_len++] = '0' + (va_num % 10);
                va_num /= 10;
            }
            for (int i = num_len - 1; i >= 0; --i)
            {
                buf_i = write_buf(buf, buf_len, buf_i, num_str[i]);
            }
            break;
        }

        case 'c':
        {
            char va_c = va_arg(ap, int);
            buf_i = write_buf(buf, buf_len, buf_i, va_c);
            break;
        }

        case 's':
        {
            const char *va_s = va_arg(ap, char *);
            if (va_s == NULL)
            {
                ret = -1;
                break;
            }

            int va_s_len = strlen(va_s);
            for (int i = 0; i < va_s_len; i++)
            {
                buf_i = write_buf(buf, buf_len, buf_i, va_s[i]);
            }
            break;
        }

        case '%':
            buf_i = write_buf(buf, buf_len, buf_i, '%');
            break;

        default:
            ret = -1;
            break;
        }

        if (buf_i == -1 || ret == -1)
        {
            break;
        }
    }

    return ret;
}

int printf(const char *fmt, ...)
{
    va_list ap;
    va_start(ap, fmt);
    int ret = _printf(BUF, BUF_SIZE, fmt, ap);

    if (ret == -1)
    {
        ret = _printf(BUF, BUF_SIZE, "<PRINTF ERROR>\n", ap);
    }
    va_end(ap);

    if (ret != -1)
    {
        ret = sys_write(FDN_STDOUT, BUF, strlen(BUF));
    }

    return ret;
}

int vsnprintf(char *buffer, size_t bufsize, const char *format, va_list arg)
{
    int ret = _printf(buffer, bufsize, format, arg);

    if (ret != -1)
    {
        ret = strlen(buffer);
    }

    return ret;
}

int snprintf(char *buff, size_t size, const char *format, ...)
{
    va_list ap;
    va_start(ap, format);
    int ret = _printf(buff, size, format, ap);
    va_end(ap);

    if (ret != -1)
    {
        ret = strlen(buff);
    }

    return ret;
}
