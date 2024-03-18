#include "../libm/libm.h"

void main(void)
{
    uint64_t ret = sys_test();
    sys_exit(ret);
}
