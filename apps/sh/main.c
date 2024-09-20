#include "../libm/libm.h"
#define BUF_LEN 128

#define MS_IN_A_DAY (24 * 60 * 60 * 1000)
#define MS_IN_A_HOUR (60 * 60 * 1000)
#define MS_IN_A_MINUTE (60 * 1000)
#define MS_IN_A_SECOND 1000

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
    else if (strcmp(cmd, "break") == 0)
    {
        sys_break();
    }
    else if (strcmp(cmd, "uptime") == 0)
    {
        uint64_t uptime_ms = sys_uptime();
        uint64_t days = uptime_ms / MS_IN_A_DAY;
        uint64_t hours = (uptime_ms % MS_IN_A_DAY) / MS_IN_A_HOUR;
        uint64_t minutes = (uptime_ms % MS_IN_A_HOUR) / MS_IN_A_MINUTE;
        uint64_t seconds = (uptime_ms % MS_IN_A_MINUTE) / MS_IN_A_SECOND;
        uint64_t milliseconds = (uptime_ms % MS_IN_A_SECOND);

        printf("%d ms\n", uptime_ms);
        printf("%d days %d hours %d minutes %d seconds %d milliseconds\n", days, hours, minutes, seconds, milliseconds);
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
        printf("\nsh$ ");
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
