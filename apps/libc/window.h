#include <stdint.h>

#define PIXEL_FORMAT_RGB 0
#define PIXEL_FORMAT_BGR 1
#define PIXEL_FORMAT_BGRA 2

typedef struct
{
    int64_t layer_id;
} WindowDescriptor;

extern WindowDescriptor *create_window(const char *title, uint64_t x_pos, uint64_t y_pos, uint64_t width, uint64_t height);
extern int destroy_window(WindowDescriptor *wdesc);
extern int add_image_to_window(WindowDescriptor *wdesc, uint32_t image_width, uint32_t image_height, uint8_t pixel_format, const char *framebuf);
