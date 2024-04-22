#include "../libm/libm.h"

void _start(void)
{
    void *ptr = sys_sbrk(1);
    sys_exit((uint64_t)ptr);
}
