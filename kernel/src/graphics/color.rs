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
    fn get_color_code(&self) -> u32;
}

impl Color for RGBColor
{
    fn get_color_code(&self) -> u32
    {
        let r = (self.r as u32) << 16;
        let g = (self.g as u32) << 8;
        let b = (self.b as u32) << 0;

        return r | g | b;
    }
}

impl Color for u32
{
    fn get_color_code(&self) -> u32 { return *self; }
}
