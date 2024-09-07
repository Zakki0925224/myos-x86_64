#include "../libm/libm.h"
#define BUF_LEN 128

static char buf[BUF_LEN] = {0};

void replace_char(char *src, const char target, const char replace)
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

void exec_cmd(const char *cmd)
{
    if (strlen(cmd) == 0)
    {
        return;
    }

    if (strcmp(cmd, "exit") == 0)
    {
        sys_exit(0);
    }
    else
    {
        printf("sh: %s: command not found\n", cmd);
    }
}

void _start()
{
    while (1)
    {
        printf("sh$ ");
        if (sys_read(0, buf, BUF_LEN) == -1)
        {
            printf("Failed to read stdin\n");
            sys_exit(1);
        }

        replace_char(buf, '\n', '\0');
        replace_char(buf, '\r', '\0');
        exec_cmd(buf);
    }

    sys_exit(0);
}
