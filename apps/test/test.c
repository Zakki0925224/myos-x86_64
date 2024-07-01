#include "../libm/libm.h"

void _start(void)
{
    printf("Hello %dworld!\n", 123456);
    sys_exit(0);
}
