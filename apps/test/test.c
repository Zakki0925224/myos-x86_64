#include "../libm/libm.h"
// static char s[30] = "helloaaaaaaaaaaaaaaaaaaaaa";

void _start(void)
{
    char s[] = "hellohellohello";
    // int r = printf("c=%c, s=%s\n", 'h', s);
    int r = printf(s);
    sys_exit(r);
}
