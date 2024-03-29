#include <stdint.h>

// syscall numbers
#define SN_WRITE 1
#define SN_TEST 3
#define SN_EXIT 4

extern uint64_t sys_write(uint16_t fd, const char *str, int len);
extern uint64_t sys_test();
extern void sys_exit(uint64_t status);
