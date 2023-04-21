pub mod color;
pub mod font;

use common::graphic_info::{GraphicInfo, PixelFormat};
use lazy_static::lazy_static;
use spin::Mutex;

use crate::arch::addr::VirtualAddress;

use self::{color::Color, font::PsfFont};

lazy_static! {
    pub static ref FRAME_BUF: Mutex<FrameBuffer> = Mutex::new(FrameBuffer::new());
}

pub struct FrameBuffer
{
    is_init: bool,
    resolution: (usize, usize),
    format: PixelFormat,
    framebuf_addr: u64,
    framebuf_size: usize,
    stride: usize,
    font: PsfFont,
}

impl FrameBuffer
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

    pub fn init(&mut self, graphic_info: GraphicInfo)
    {
        self.resolution = (graphic_info.resolution.0 as usize, graphic_info.resolution.1 as usize);
        self.format = graphic_info.format;
        self.framebuf_addr = graphic_info.framebuf_addr;
        self.framebuf_size = graphic_info.framebuf_size as usize;
        self.stride = graphic_info.stride as usize;
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

    pub fn set_color(&self, x: usize, y: usize, color: &impl Color)
    {
        self.write_pixel(x, y, color.get_color_code(self.get_pixel_format()));
    }

    pub fn draw_rect(&self, x1: usize, y1: usize, width: usize, height: usize, color: &impl Color)
    {
        for y in y1..y1 + height
        {
            for x in x1..x1 + width
            {
                self.set_color(x, y, color);
            }
        }
    }

    pub fn draw_font(&self, x1: usize, y1: usize, c: char, color: &impl Color)
    {
        if let Some(glyph) = self.font.get_glyph(self.font.unicode_char_to_glyph_index(c))
        {
            for h in 0..self.font.get_height()
            {
                for w in 0..self.font.get_width()
                {
                    if (glyph[h] << w) & 0x80 == 0x80
                    {
                        self.draw_rect(x1 + w, y1 + h, 1, 1, color);
                    }
                }
            }
        }
        else
        {
            panic!("Failed to get font glyph");
        }
    }

    pub fn copy_pixel(&self, x: usize, y: usize, to_x: usize, to_y: usize)
    {
        let data = self.read_pixel(x, y);
        self.write_pixel(to_x, to_y, data);
    }

    pub fn clear(&self, color: &impl Color)
    {
        let (max_x, max_y) = self.get_resolution();
        for y in 0..max_y
        {
            for x in 0..max_x
            {
                self.set_color(x, y, color);
            }
        }
    }

    fn read_pixel(&self, x: usize, y: usize) -> u32
    {
        if !self.is_init
        {
            panic!("Graphics is not initialized");
        }

        let (res_x, res_y) = self.get_resolution();
        if x >= res_x || y >= res_y
        {
            panic!("Outside the frame buffer area was specified");
        }

        let addr = VirtualAddress::new(self.framebuf_addr + 4 * (res_x * y) as u64 + 4 * x as u64);
        return addr.read_volatile();
    }

    fn write_pixel(&self, x: usize, y: usize, data: u32)
    {
        if !self.is_init
        {
            panic!("Graphics is not initialized");
        }

        let (res_x, res_y) = self.get_resolution();
        if x >= res_x || y >= res_y
        {
            panic!("Outside the frame buffer area was specified");
        }

        let addr = VirtualAddress::new(self.framebuf_addr + 4 * (res_x * y) as u64 + 4 * x as u64);
        addr.write_volatile(data);
    }
}
