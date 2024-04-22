#include "../libm/libm.h"

void _start(int argc, char *argv[])
{
    struct utsname *buf = sys_sbrk(sizeof(struct utsname));
    sys_uname(buf);

    if (argc == 1)
    {
        printf(buf->sysname);
        sys_exit(0);
    }

    if (strcmp(argv[1], "--help") == 0)
    {
        printf("Usage: uname [OPTION]...\n");
        printf("Print certain system information. With no OPTION, same as -s.\n\n");
        printf(" -a\tprint all information\n");
        printf(" -s\tprint the kernel name\n");
        printf(" -n\tprint the network node hostname\n");
        printf(" -r\tprint the kernel release\n");
        printf(" -v\tprint the kernel version\n");
        printf(" -m\tprint the machine hardware name\n");
        printf(" -d\tprint the domain name\n");
        sys_exit(0);
    }

    for (int i = 1; i < argc; i++)
    {
        if (strcmp(argv[i], "-a") == 0)
        {
            printf(buf->sysname);
            printf(" ");
            printf(buf->nodename);
            printf(" ");
            printf(buf->release);
            printf(" ");
            printf(buf->version);
            printf(" ");
            printf(buf->machine);
            printf(" ");
            printf(buf->domainname);
            sys_exit(0);
        }
    }

    for (int i = 1; i < argc; i++)
    {
        if (strcmp(argv[i], "-s") == 0)
        {
            printf(buf->sysname);
            printf(" ");
        }
        else if (strcmp(argv[i], "-n") == 0)
        {
            printf(buf->nodename);
            printf(" ");
        }
        else if (strcmp(argv[i], "-r") == 0)
        {
            printf(buf->release);
            printf(" ");
        }
        else if (strcmp(argv[i], "-v") == 0)
        {
            printf(buf->version);
            printf(" ");
        }
        else if (strcmp(argv[i], "-m") == 0)
        {
            printf(buf->machine);
            printf(" ");
        }
        else if (strcmp(argv[i], "-d") == 0)
        {
            printf(buf->domainname);
            printf(" ");
        }
    }

    sys_exit(0);
}
