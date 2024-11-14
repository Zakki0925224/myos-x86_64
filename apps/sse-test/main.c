#include <stdio.h>

float hoge()
{
    return 0;
}

int main()
{
    // __m128 a = _mm_set_ps(1.0f, 2.0f, 3.0f, 4.0f);
    // __m128 b;

    // __asm__(
    //     "movaps %[a], %[b]"
    //     : [b] "=x"(b)
    //     : [a] "x"(a));

    // float result[4];
    // _mm_store_ps(result, b);
    // printf("Result: %.2f, %.2f, %.2f, %.2f\n", result[0], result[1], result[2], result[3]);
    printf("%d\n", hoge());
    return 0;
}
