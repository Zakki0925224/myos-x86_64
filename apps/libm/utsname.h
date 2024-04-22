#define UTS_LEN 64

struct utsname
{
    char sysname[UTS_LEN];
    char nodename[UTS_LEN];
    char release[UTS_LEN];
    char version[UTS_LEN];
    char machine[UTS_LEN];
    char domainname[UTS_LEN];
};
