#include "../libm/libm.h"

void _start(void)
{
    char *str_buf = (char *)malloc(4096);
    if (str_buf == NULL)
    {
        sys_exit(1);
    }

    if (sys_read(0, str_buf, 5) == -1)
    {
        sys_exit(2);
    }

    printf("\"%s\"\n", str_buf);
    sys_exit(0);
}
