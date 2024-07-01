#include "../libm/libm.h"

void _start(int argc, char *argv[])
{
    for (int i = 1; i < argc; i++)
    {
        printf("\"%s\" ", argv[i]);
    }
    printf("\n");
    sys_exit(0);
}
