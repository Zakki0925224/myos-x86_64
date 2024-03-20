#include "../libm/libm.h"

void _start(void)
{
    uint64_t ret = sys_test();
    sys_exit(ret);
}
