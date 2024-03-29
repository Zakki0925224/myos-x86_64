#include "../libm/libm.h"

void _start(int argc, char *argv[])
{
    if (strcmp(argv[1], argv[2]) != 0)
    {
        sys_exit(1);
    }

    sys_exit((uint64_t)strlen(argv[1]));
}
