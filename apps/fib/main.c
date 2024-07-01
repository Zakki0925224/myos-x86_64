#include "../libm/libm.h"

int fib(int n)
{
    if (n <= 1)
    {
        return n;
    }

    return fib(n - 1) + fib(n - 2);
}

void _start(void)
{
    for (int i = 0; i < 50; i++)
    {
        printf("%d, ", fib(i));
    }
    printf("\n");

    sys_exit(0);
}
