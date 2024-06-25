#include "../libm/libm.h"

char *itoa(int n)
{
    static char buffer[32];
    int i = 30;

    do
    {
        buffer[i--] = n % 10 + '0';
        n = n / 10;
    } while (n > 0);

    return &buffer[i + 1];
}

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
        printf(itoa(fib(i)));
        printf(", ");
    }
    printf("\n");

    sys_exit(0);
}
