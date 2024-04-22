#include "../libm/libm.h"

void _start(int argc, char *argv[])
{
    struct utsname *buf = sys_sbrk(sizeof(struct utsname));
    sys_uname(buf);
    printf(buf->sysname);
    sys_exit(0);
}
