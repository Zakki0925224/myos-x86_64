#include "../libm/libm.h"

void _start(void)
{
    char s[14] = "Hello world!\n";
    // char s_len = strlen(s);
    //  sys_write(FDN_STDOUT, s, s_len);
    printf(s);
    sys_exit(0);
}
