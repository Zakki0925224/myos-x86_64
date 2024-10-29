#include "../libm/libm.h"
#include <emmintrin.h>

#define N 8

void add_vectors(const float *a, const float *b, float *result, int n)
{
    for (int i = 0; i < n; i += 4)
    {
        __m128 vec_a = _mm_loadu_ps(&a[i]);
        __m128 vec_b = _mm_loadu_ps(&b[i]);

        __m128 vec_result = _mm_add_ps(vec_a, vec_b);

        _mm_storeu_ps(&result[i], vec_result);
    }
}

int main()
{
    float a[N] = {1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0};
    float b[N] = {0.5, 1.5, 2.5, 3.5, 4.5, 5.5, 6.5, 7.5};
    float result[N];

    add_vectors(a, b, result, N);

    for (int i = 0; i < N; ++i)
    {
        printf("result[%d] = %f\n", i, result[i]);
    }

    return 0;
}
