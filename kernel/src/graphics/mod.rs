pub mod color;
pub mod font;

use core::ptr::write_volatile;

use common::graphic_info::PixelFormat;
use lazy_static::lazy_static;
use spin::Mutex;

use self::{color::Color, font::PsfFont};

lazy_static! {
    pub static ref GRAPHICS: Mutex<Graphics> = Mutex::new(Graphics::new());
}

pub struct Graphics
{
    is_init: bool,
    resolution: (usize, usize),
    format: PixelFormat,
    framebuf_addr: u64,
    framebuf_size: usize,
    stride: usize,
    font: PsfFont,
}

impl Graphics
{
    pub fn new() -> Self
    {
        return Self {
            is_init: false,
            resolution: (0, 0),
            format: PixelFormat::Rgb,
            framebuf_addr: 0,
            framebuf_size: 0,
            stride: 0,
            font: PsfFont::new(),
        };
    }

    pub fn init(
        &mut self,
        resolution: (usize, usize),
        format: PixelFormat,
        framebuf_addr: u64,
        framebuf_size: usize,
        stride: usize,
    )
    {
        self.resolution = resolution;
        self.format = format;
        self.framebuf_addr = framebuf_addr;
        self.framebuf_size = framebuf_size;
        self.stride = stride;
        self.is_init = true;
    }

    pub fn is_init(&self) -> bool { return self.is_init; }

    pub fn get_resolution(&self) -> (usize, usize) { return self.resolution; }

    pub fn get_stride(&self) -> usize { return self.stride; }

    pub fn get_pixel_format(&self) -> PixelFormat { return self.format; }

    pub fn get_font_glyph_size(&self) -> (usize, usize)
    {
        return (self.font.get_width(), self.font.get_width());
    }

    pub fn set_color(&self, x: usize, y: usize, color: &impl Color) -> Result<(), &str>
    {
        if !self.is_init
        {
            return Err("Graphics is not initialized");
        }

        let (res_x, res_y) = self.get_resolution();

        if x > res_x || y > res_y
        {
            return Err("Outside the frame buffer area was specified");
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
        if !self.is_init
        {
            return Err("Graphics is not initialized");
        }

        for y in y1..y1 + height
        {
            for x in x1..x1 + width
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

    pub fn draw_font(&self, x1: usize, y1: usize, c: char, color: &impl Color) -> Result<(), &str>
    {
        if !self.is_init
        {
            return Err("Graphics is not initialized");
        }

        if let Some(glyph) = self.font.get_glyph(self.font.unicode_char_to_glyph_index(c))
        {
            for h in 0..self.font.get_height()
            {
                for w in 0..self.font.get_width()
                {
                    if (glyph[h] << w) & 0x80 == 0x80
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

    pub fn clear(&self, color: &impl Color) -> Result<(), &str>
    {
        if !self.is_init
        {
            return Err("Graphics is not initialized");
        }

        let (max_x, max_y) = self.get_resolution();
        for y in 0..max_y
        {
            for x in 0..max_x
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
}
