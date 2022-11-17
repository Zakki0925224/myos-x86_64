pub mod color;
pub mod font;

use core::ptr::write_volatile;

use common::graphic_info::PixelFormat;

use self::{color::Color, font::PsfFont};

pub struct Graphics
{
    resolution: (usize, usize),
    format: PixelFormat,
    framebuf_addr: u64,
    framebuf_size: usize,
    stride: usize,
    font: PsfFont,
}

impl Graphics
{
    pub fn new(
        resolution: (usize, usize),
        format: PixelFormat,
        framebuf_addr: u64,
        framebuf_size: usize,
        stride: usize,
    ) -> Self
    {
        return Self {
            resolution,
            format,
            framebuf_addr,
            framebuf_size,
            stride,
            font: PsfFont::new(),
        };
    }

    pub fn get_resolution(&self) -> (usize, usize) { return self.resolution; }

    pub fn get_pixel_format(&self) -> PixelFormat { return self.format; }

    pub fn set_color(&self, x: usize, y: usize, color: &impl Color) -> Result<(), &str>
    {
        let (res_x, res_y) = self.get_resolution();

        if x > res_x || y > res_y
        {
            return Err("Outside the frame buffer area was specified.");
        }

        unsafe {
            let ptr = (self.framebuf_addr + 4 * (res_x * y) as u64 + 4 * x as u64) as *mut u32;
            write_volatile(ptr, color.get_color_code(self.get_pixel_format()));
        }

        return Ok(());
    }

    pub fn draw_rect(
        &self,
        x1: usize,
        y1: usize,
        width: usize,
        height: usize,
        color: &impl Color,
    ) -> Result<(), &str>
    {
        for y in y1..=y1 + height
        {
            for x in x1..=x1 + width
            {
                if let Err(msg) =
                    self.set_color(x, y, &color.get_color_code(self.get_pixel_format()))
                {
                    return Err(msg);
                }
            }
        }

        return Ok(());
    }

    // based vga font
    pub fn draw_font(&self, x1: usize, y1: usize, c: char, color: &impl Color) -> Result<(), &str>
    {
        if let Some(glyph) = self.font.get_glyph(5)
        {
            for h in 0..self.font.height
            {
                for w in 0..self.font.width
                {
                    let bit = glyph[h * self.font.width / 8] >> self.font.width - 1 - w;

                    if bit == 1
                    {
                        if let Err(msg) = self.draw_rect(x1 + w, y1 + h, 1, 1, color)
                        {
                            return Err(msg);
                        }
                    }
                }
            }

            return Ok(());
        }
        else
        {
            return Err("Failed to get font glyph");
        }
    }

    pub fn clear(&self, color: &impl Color)
    {
        let (max_x, max_y) = self.get_resolution();
        for y in 0..max_y
        {
            for x in 0..max_x
            {
                self.set_color(x, y, &color.get_color_code(self.get_pixel_format())).unwrap();
            }
        }
    }
}
