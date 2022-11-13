pub mod color;

use core::ptr::write_volatile;

use common::graphic_info::GraphicInfo;

use self::color::Color;

pub struct Graphics
{
    graphic_info: GraphicInfo,
}

impl Graphics
{
    pub fn new(graphic_info: GraphicInfo) -> Self { return Self { graphic_info }; }

    pub fn get_resolution(&self) -> (usize, usize)
    {
        let res = self.graphic_info.resolution;
        return (res.0 as usize, res.1 as usize);
    }

    pub fn set_color(&self, x: usize, y: usize, color: impl Color) -> Result<(), &str>
    {
        let (res_x, res_y) = self.get_resolution();

        if x > res_x || y > res_y
        {
            return Err("Outside the frame buffer area was specified.");
        }

        unsafe {
            let ptr = (self.graphic_info.framebuf_addr + 4 * (res_x * y) as u64 + 4 * x as u64)
                as *mut u32;
            write_volatile(ptr, color.get_color_code());
        }

        return Ok(());
    }

    pub fn draw_rect(
        &self,
        x1: usize,
        y1: usize,
        width: usize,
        height: usize,
        color: impl Color,
    ) -> Result<(), &str>
    {
        let color_code = color.get_color_code();

        for y in y1..=y1 + height
        {
            for x in x1..=x1 + width
            {
                if let Err(msg) = self.set_color(x, y, color_code)
                {
                    return Err(msg);
                }
            }
        }

        return Ok(());
    }

    pub fn clear(&self, color: impl Color)
    {
        let (max_x, max_y) = self.get_resolution();
        let color_code = color.get_color_code();

        for y in 0..max_y
        {
            for x in 0..max_x
            {
                self.set_color(x, y, color_code).unwrap();
            }
        }
    }
}
