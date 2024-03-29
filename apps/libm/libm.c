#include <stdint.h>
#include "syscalls.h"
#include "string.h"

// syscalls.h
static uint64_t syscall(uint64_t syscall_number, uint64_t arg1, uint64_t arg2, uint64_t arg3, uint64_t arg4, uint64_t arg5)
{
    uint64_t ret_val;
    __asm__ volatile(
        "movq %1, %%rdi\n"
        "movq %2, %%rsi\n"
        "movq %3, %%rdx\n"
        "movq %4, %%r10\n"
        "movq %5, %%r8\n"
        "movq %6, %%r9\n"
        "syscall\n"
        "movq %%rax, %0\n"
        : "=r"(ret_val)
        : "r"(syscall_number), "r"(arg1), "r"(arg2), "r"(arg3), "r"(arg4), "r"(arg5)
        : "rdi", "rsi", "rdx", "r10", "r8", "r9", "memory");
    return ret_val;
}

uint64_t sys_write(uint16_t fd, const char *str, int len)
{
    return syscall(SN_WRITE, (uint64_t)fd, (uint64_t)str, (uint64_t)len, 0, 0);
}

uint64_t sys_test()
{
    return syscall(SN_TEST, 0, 0, 0, 0, 0);
}

void sys_exit(uint64_t status)
{
    syscall(SN_EXIT, status, 0, 0, 0, 0);
}

// string.h
int strcmp(const char *s1, const char *s2)
{
    while (*s1 != '\0' && *s2 != '\0' && *s1 == *s2)
    {
        s1++;
        s2++;
    }
    return *s1 - *s2;
}

int strlen(const char *str)
{
    int res = 0;
    while (*str++)
    {
        res++;
    }
    return res;
}
