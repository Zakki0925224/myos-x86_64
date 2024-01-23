int main(void)
{
    asm volatile("movl $0xb, %eax");
    asm volatile("movq %rcx, %r10");
    asm volatile("syscall");
    return 0;
}
