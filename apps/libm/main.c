#include "libm.h"

void _start(int argc, char const *argv[])
{
    exit((uint64_t)main(argc, argv));
}
