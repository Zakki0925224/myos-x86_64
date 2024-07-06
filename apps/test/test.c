#include "../libm/libm.h"
static char s[30] = "helloaaaaaaaaaaaaaaaaaaaaa";

void _start(void)
{
    // char s[30] = "helloaaaaaaaaaaaaaaaaaaa";
    int r = printf("c=%c, s=%s\n", 'h', s);
    sys_exit(r);
}
