#include "../libm/libm.h"

void _start()
{
    int64_t fd = sys_open("/mnt/initramfs/test.txt");
    if (fd == -1)
    {
        sys_exit(1);
    }

    stat *f_stat = (stat *)malloc(sizeof(stat));
    if (sys_stat(fd, f_stat) == -1)
    {
        sys_exit(1);
    }
    printf("file size: %d bytes\n", f_stat->size);

    sys_exit(0);
}
