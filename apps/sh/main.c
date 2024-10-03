#include "../libm/libm.h"
#define BUF_LEN 128

#define MS_IN_A_DAY (24 * 60 * 60 * 1000)
#define MS_IN_A_HOUR (60 * 60 * 1000)
#define MS_IN_A_MINUTE (60 * 1000)
#define MS_IN_A_SECOND 1000

static char buf[BUF_LEN] = {0};
static char *splitted_buf[BUF_LEN];
static char cwd_path[BUF_LEN] = {0};

void exec_cmd(const char *cmd)
{
    int cmdargs_len = split(cmd, ' ', splitted_buf, BUF_LEN);

    if (cmdargs_len < 1)
    {
        return;
    }

    if (strlen(splitted_buf[0]) == 0)
    {
        return;
    }

    if (strcmp(splitted_buf[0], "exit") == 0)
    {
        sys_exit(0);
    }
    else if (strcmp(splitted_buf[0], "break") == 0)
    {
        sys_break();
    }
    else if (strcmp(splitted_buf[0], "cd") == 0)
    {
        if (cmdargs_len < 2)
        {
            return;
        }

        if (sys_chdir(splitted_buf[1]) == -1)
        {
            printf("sh: cd: failed to change directory\n");
            return;
        }
    }
    else if (strcmp(splitted_buf[0], "uptime") == 0)
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
    else if (strcmp(splitted_buf[0], "exec") == 0)
    {
        if (cmdargs_len < 2)
        {
            printf("sh: exec: missing argument\n");
            return;
        }

        char *args = splitted_buf[1];
        if (cmdargs_len > 2)
        {
            args = concatenate(splitted_buf + 1, cmdargs_len - 1, " ");

            if (args == NULL)
            {
                printf("sh: exec: failed to concatenate arguments\n");
                return;
            }
        }

        if (sys_exec(args) == -1)
        {
            printf("sh: exec: failed to execute\n");
            return;
        }
    }
    else
    {
        printf("sh: %s: command not found\n", cmd);
    }
}

void _start()
{
    int getcwd_ret;

    while (1)
    {
        getcwd_ret = sys_getcwd(cwd_path, sizeof(cwd_path));
        printf("\nsh[%s]$ ", getcwd_ret == -1 ? "UNKNOWN" : cwd_path);

        if (sys_read(0, buf, BUF_LEN) == -1)
        {
            printf("Failed to read stdin\n");
            sys_exit(1);
        }

        replace(buf, '\n', '\0');
        replace(buf, '\r', '\0');
        exec_cmd(buf);
    }

    sys_exit(0);
}
