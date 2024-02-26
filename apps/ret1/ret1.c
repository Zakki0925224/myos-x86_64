static void _exit(long status)
{
    __asm__ volatile(
        "movq %0, %%rax \n\t"
        "movq %1, %%rdi \n\t"
        "syscall        \n\t"
        :
        : "i"(60), "r"(status)
        : "%rax", "%rdi");
}

void main(void)
{
    _exit(1);
}
