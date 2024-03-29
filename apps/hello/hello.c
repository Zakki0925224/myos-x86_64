#include "../libm/libm.h"

void _start(void)
{
    char s[13] = "Hello world!\n";
    char s_len = strlen(s);
    sys_write(1, s, s_len);
    sys_exit(0);
}
