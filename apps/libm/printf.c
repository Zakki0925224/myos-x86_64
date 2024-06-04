#include "libm.h"

int printf(const char *str)
{
    int str_len = strlen(str);
    int64_t ret_val = sys_write(FDN_STDOUT, str, str_len);

    if (ret_val == -1)
    {
        return -1;
    }

    return 0;
}
