#include <stdio.h>
#include <stat.h>
#include <window.h>
#include <string.h>
#include <syscalls.h>

#define BUF_LEN 128

#define MS_IN_A_DAY (24 * 60 * 60 * 1000)
#define MS_IN_A_HOUR (60 * 60 * 1000)
#define MS_IN_A_MINUTE (60 * 1000)
#define MS_IN_A_SECOND 1000

static char buf[BUF_LEN] = {0};
static char *splitted_buf[BUF_LEN];
static char cwd_path[BUF_LEN] = {0};
static char cwdenames[BUF_LEN * 10] = {0};

void exec_cmd(char *cmd)
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
    else if (strcmp(splitted_buf[0], "ls") == 0)
    {
        if (sys_getcwdenames(cwdenames, sizeof(cwdenames)) == -1)
        {
            printf("sh: ls: failed to get entry names in the current working directory\n");
            return;
        }

        char old_c = '\0';

        for (int i = 0; i < (int)sizeof(cwdenames); i++)
        {
            char c = cwdenames[i];

            // end of name list
            if (old_c == '\0' && c == '\0' && i > 0)
            {
                break;
            }

            if (c == '\0')
            {
                printf("  ");
            }
            else
            {
                printf("%c", c);
            }

            old_c = cwdenames[i];

            // clear
            cwdenames[i] = '\0';
        }
    }
    else if (strcmp(splitted_buf[0], "cat") == 0)
    {
        if (cmdargs_len < 2)
        {
            return;
        }

        char *filepath = splitted_buf[1];
        int64_t fd = sys_open(filepath);

        if (fd == -1)
        {
            printf("sh: cat: failed to open the file\n");
            return;
        }

        f_stat *file_stat = (f_stat *)malloc(sizeof(f_stat));
        if (sys_stat(fd, file_stat) == -1)
        {
            printf("sh: cat: failed to get the file status\n");
            return;
        }

        char *f_buf = (char *)malloc(file_stat->size);
        if (sys_read(fd, f_buf, file_stat->size) == -1)
        {
            printf("sh: cat: failed to read the file\n");
            return;
        }

        if (sys_close(fd) == -1)
        {
            printf("sh: cat: failed to close the file\n");
            return;
        }

        printf("%s\n", f_buf);
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
            args = concatenate((const char **)(splitted_buf + 1), cmdargs_len - 1, " ");

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
    else if (strcmp(splitted_buf[0], "window") == 0)
    {
        WindowDescriptor *wdesc = create_window("test window", 200, 50, 300, 200);
        if (wdesc == NULL)
        {
            printf("sh: window: failed to create window\n");
            return;
        }
    }
    else
    {
        printf("sh: %s: command not found\n", cmd);
    }
}

int main(void)
{
    int getcwd_ret;

    while (1)
    {
        getcwd_ret = sys_getcwd(cwd_path, sizeof(cwd_path));
        printf("\n[%s]$ ", getcwd_ret == -1 ? "UNKNOWN" : cwd_path);

        if (sys_read(0, buf, BUF_LEN) == -1)
        {
            printf("Failed to read stdin\n");
            sys_exit(1);
        }

        replace(buf, '\n', '\0');
        replace(buf, '\r', '\0');
        exec_cmd(buf);
    }

    return 0;
}
