#include "../libm/libm.h"

void _start()
{
    int64_t fd = sys_open("/mnt/initramfs/test.txt");
    if (fd == -1)
    {
        sys_exit(1);
    }

    void *buf = malloc(4096);
    if (buf == NULL)
    {
        sys_exit(2);
    }

    if (sys_read(fd, buf, 4096) == -1)
    {
        sys_exit(3);
    }

    char *str_buf = (char *)buf;
    if (str_buf[0] != 'h' && str_buf[1] != 'e' && str_buf[2] != 'l' && str_buf[3] != 'l' && str_buf[4] != 'o')
    {
        sys_exit(4);
    }

    if (sys_close(fd) == -1)
    {
        sys_exit(4);
    }

    sys_exit(0);
}
