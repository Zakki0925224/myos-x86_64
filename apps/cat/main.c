#include <stdio.h>
#include <stdlib.h>
#include <stat.h>
#include <syscalls.h>

int main(int argc, char *argv[])
{
    if (argc < 2)
    {
        return 0;
    }

    int64_t fd = sys_open(argv[1]);

    if (fd == -1)
    {
        printf("cat: failed to open the file\n");
        return 1;
    }

    f_stat *file_stat = (f_stat *)malloc(sizeof(f_stat));
    if (sys_stat(fd, file_stat) == -1)
    {
        printf("cat: failed to get the file status\n");
        return 1;
    }

    char *f_buf = (char *)malloc(file_stat->size);
    if (sys_read(fd, f_buf, file_stat->size) == -1)
    {
        printf("cat: failed to read the file\n");
        return 1;
    }

    if (sys_close(fd) == -1)
    {
        printf("cat: failed to close the file\n");
        return 1;
    }

    printf("%s\n", f_buf);
    return 0;
}
