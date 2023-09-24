use common::graphic_info::PixelFormat;

pub const COLOR_WHITE: RgbColor = RgbColor::new(255, 255, 255);
pub const COLOR_OLIVE: RgbColor = RgbColor::new(128, 128, 0);
pub const COLOR_YELLOW: RgbColor = RgbColor::new(255, 255, 0);
pub const COLOR_FUCHSIA: RgbColor = RgbColor::new(255, 0, 255);
pub const COLOR_SILVER: RgbColor = RgbColor::new(192, 192, 192);
pub const COLOR_CYAN: RgbColor = RgbColor::new(0, 255, 255);
pub const COLOR_GREEN: RgbColor = RgbColor::new(0, 255, 0);
pub const COLOR_RED: RgbColor = RgbColor::new(255, 0, 0);
pub const COLOR_GRAY: RgbColor = RgbColor::new(128, 128, 128);
pub const COLOR_BLUE: RgbColor = RgbColor::new(0, 0, 255);
pub const COLOR_PURPLE: RgbColor = RgbColor::new(128, 0, 128);
pub const COLOR_BLACK: RgbColor = RgbColor::new(0, 0, 0);
pub const COLOR_NAVY: RgbColor = RgbColor::new(0, 0, 128);
pub const COLOR_TEAL: RgbColor = RgbColor::new(0, 128, 128);
pub const COLOR_MAROON: RgbColor = RgbColor::new(128, 0, 0);

#[derive(Debug, Clone, Copy)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        return RgbColor { r, g, b };
    }
}

impl From<(u8, u8, u8)> for RgbColor {
    fn from(color: (u8, u8, u8)) -> Self {
        return RgbColor::new(color.0, color.1, color.2);
    }
}

pub trait Color {
    fn get_color_code(&self, pixel_format: PixelFormat) -> u32;
}

impl Color for RgbColor {
    fn get_color_code(&self, pixel_format: PixelFormat) -> u32 {
        let r = self.r as u32;
        let g = self.g as u32;
        let b = self.b as u32;

        // only support bgr or rgb pixel format
        if pixel_format == PixelFormat::Bgr {
            return r << 16 | g << 8 | b << 0;
        } else {
            return b << 16 | g << 8 | r << 0;
        }
    }
}

impl Color for u32 {
    fn get_color_code(&self, _: PixelFormat) -> u32 {
        return *self;
    }
}
