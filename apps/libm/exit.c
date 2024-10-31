#include "libm.h"

void exit(int status)
{
    sys_exit((uint64_t)status);
}
