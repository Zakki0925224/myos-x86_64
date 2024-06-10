#include "../libm/libm.h"

#define MEM_LEN 30000

static uint8_t mem[MEM_LEN] = {0};

// int exec_bf(const char *bf_code)
// {
//     int code_len = strlen(bf_code);
//     char str[2];
//     // instruction pointer
//     int ip = 0;
//     // memory pointer register
//     int pr = 0;
//     while (1)
//     {
//         if (code_len <= ip)
//         {
//             break;
//         }

//         switch (bf_code[ip])
//         {
//         case '+':
//             mem[pr]++;
//             break;
//         case '.':
//             str[0] = (char)mem[pr];
//             str[1] = '\0';
//             printf(str);
//             break;

//         default:
//             printf("Unknown instruction\n");
//             return -1;
//         }

//         ip++;
//     }

//     return 0;
// }

void _start(int argc, char *argv[])
{
    char *bf_code = "+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++.";

    printf("Welcome to Brainf**k compiler!\n");

    int code_len = strlen(bf_code);
    char str[2];
    // instruction pointer
    int ip = 0;
    // memory pointer register
    int pr = 0;
    while (1)
    {
        if (code_len <= ip)
        {
            break;
        }

        switch (bf_code[ip])
        {
        case '+':
            mem[pr]++;
            break;
        case '.':
            str[0] = (char)mem[pr];
            str[1] = '\0';
            printf(str);
            break;

        default:
            printf("Unknown instruction\n");
            sys_exit(-1);
            break; // unreachable
        }

        ip++;
    }

    sys_exit(0);
}
