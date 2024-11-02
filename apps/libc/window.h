#include <stdint.h>

typedef struct
{
    int64_t layer_id;
} WindowDescriptor;

extern WindowDescriptor *create_window(const char *title, uint64_t x_pos, uint64_t y_pos, uint64_t width, uint64_t height);
extern int destroy_window(WindowDescriptor *wdesc);
