#include <stdint.h>
#include "libm.h"

static int64_t syscall(uint64_t syscall_number, uint64_t arg1, uint64_t arg2, uint64_t arg3, uint64_t arg4, uint64_t arg5)
{
    int64_t ret_val;
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

int64_t sys_read(int64_t fd, void *buf, int buf_len)
{
    return syscall(SN_READ, (uint64_t)fd, (uint64_t)buf, (uint64_t)buf_len, 0, 0);
}

int64_t sys_write(int64_t fd, const char *str, int len)
{
    return syscall(SN_WRITE, (uint64_t)fd, (uint64_t)str, (uint64_t)len, 0, 0);
}

int64_t sys_open(const char *filepath)
{
    return syscall(SN_OPEN, (uint64_t)filepath, 0, 0, 0, 0);
}

int64_t sys_close(int64_t fd)
{
    return syscall(SN_CLOSE, (uint64_t)fd, 0, 0, 0, 0);
}

void sys_exit(uint64_t status)
{
    syscall(SN_EXIT, status, 0, 0, 0, 0);
}

void *sys_sbrk(uint64_t len)
{
    int64_t addr = syscall(SN_SBRK, len, 0, 0, 0, 0);
    return (void *)addr;
}

int64_t sys_uname(utsname *buf)
{
    return syscall(SN_UNAME, (uint64_t)buf, 0, 0, 0, 0);
}

void sys_break(void)
{
    syscall(SN_BREAK, 0, 0, 0, 0, 0);
}
