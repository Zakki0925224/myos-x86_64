#include "../libm/libm.h"

void _start(void)
{
    // char s[18] = "hoge huga hogera\0";
    printf("c=%c, s=%s\n", 'h', "hoge huga hogera");
    sys_exit(0);
}
