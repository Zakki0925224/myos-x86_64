#include "stdlib.h"
#include "syscalls.h"
#include "window.h"

WindowDescriptor *create_window(const char *title, uint64_t x_pos, uint64_t y_pos, uint64_t width, uint64_t height)
{
    int64_t layer_id = sys_create_window(title, x_pos, y_pos, width, height);

    if (layer_id == -1)
    {
        return NULL;
    }

    WindowDescriptor *wdesc = (WindowDescriptor *)malloc(sizeof(WindowDescriptor));
    wdesc->layer_id = layer_id;
    return wdesc;
}

int destroy_window(WindowDescriptor *wdesc)
{
    if (wdesc == NULL)
    {
        return -1;
    }

    return (int)sys_destroy_window(wdesc->layer_id);
}

int flush_window(WindowDescriptor *wdesc)
{
    if (wdesc == NULL)
    {
        return -1;
    }

    return (int)sys_flush_window(wdesc->layer_id);
}

int add_image_to_window(WindowDescriptor *wdesc, uint32_t image_width, uint32_t image_height, uint8_t pixel_format, const char *framebuf)
{
    if (wdesc == NULL)
    {
        return -1;
    }

    return (int)sys_add_image_to_window(wdesc->layer_id, (uint64_t)image_width, (uint64_t)image_height, pixel_format, framebuf);
}
