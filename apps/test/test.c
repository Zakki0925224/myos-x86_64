#include "../libm/libm.h"

void _start(void)
{
    char s[] = "hellohellohello";
    int r = printf("c=%c, s=%s\n", 'h', s);
    sys_exit(r);
}
