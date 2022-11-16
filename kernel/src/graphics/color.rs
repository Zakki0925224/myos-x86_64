use common::graphic_info::PixelFormat;

#[derive(Clone, Copy)]
pub struct RGBColor
{
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RGBColor
{
    pub fn new(r: u8, g: u8, b: u8) -> Self { return RGBColor { r, g, b }; }
}

pub trait Color
{
    fn get_color_code(&self, pixel_format: PixelFormat) -> u32;
}

impl Color for RGBColor
{
    fn get_color_code(&self, pixel_format: PixelFormat) -> u32
    {
        // only support bgr or rgb pixel format
        if pixel_format == PixelFormat::Bgr
        {
            return (self.r as u32) << 16 | (self.g as u32) << 8 | (self.b as u32) << 0;
        }
        else
        {
            return (self.b as u32) << 16 | (self.g as u32) << 8 | (self.r as u32) << 0;
        }
    }
}

impl Color for u32
{
    fn get_color_code(&self, _: PixelFormat) -> u32 { return *self; }
}

// TODO: const RGBColor not working (filled by all 1)
