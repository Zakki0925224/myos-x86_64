#include <stdio.h>

int fib(int n)
{
    if (n <= 1)
    {
        return n;
    }

    return fib(n - 1) + fib(n - 2);
}

int main(void)
{
    for (int i = 0; i < 50; i++)
    {
        printf("%d, ", fib(i));
    }
    printf("\n");

    return 0;
}
