#include "libm.h"

void *malloc(size_t len)
{
    return sys_sbrk(len);
}
