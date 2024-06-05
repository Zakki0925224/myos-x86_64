use common::graphic_info::PixelFormat;

pub const COLOR_WHITE: RgbColorCode = RgbColorCode::new(255, 255, 255);
pub const COLOR_OLIVE: RgbColorCode = RgbColorCode::new(128, 128, 0);
pub const COLOR_YELLOW: RgbColorCode = RgbColorCode::new(255, 255, 0);
pub const COLOR_FUCHSIA: RgbColorCode = RgbColorCode::new(255, 0, 255);
pub const COLOR_SILVER: RgbColorCode = RgbColorCode::new(192, 192, 192);
pub const COLOR_CYAN: RgbColorCode = RgbColorCode::new(0, 255, 255);
pub const COLOR_GREEN: RgbColorCode = RgbColorCode::new(0, 255, 0);
pub const COLOR_RED: RgbColorCode = RgbColorCode::new(255, 0, 0);
pub const COLOR_GRAY: RgbColorCode = RgbColorCode::new(128, 128, 128);
pub const COLOR_BLUE: RgbColorCode = RgbColorCode::new(0, 0, 255);
pub const COLOR_PURPLE: RgbColorCode = RgbColorCode::new(128, 0, 128);
pub const COLOR_BLACK: RgbColorCode = RgbColorCode::new(0, 0, 0);
pub const COLOR_NAVY: RgbColorCode = RgbColorCode::new(0, 0, 128);
pub const COLOR_TEAL: RgbColorCode = RgbColorCode::new(0, 128, 128);
pub const COLOR_MAROON: RgbColorCode = RgbColorCode::new(128, 0, 0);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct RgbColorCode {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl From<(u8, u8, u8)> for RgbColorCode {
    fn from(color: (u8, u8, u8)) -> Self {
        Self {
            r: color.0,
            g: color.1,
            b: color.2,
        }
    }
}

impl RgbColorCode {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn from_pixel_data(data: u32, pixel_format: PixelFormat) -> Self {
        match pixel_format {
            PixelFormat::Bgr => Self {
                r: (data >> 16) as u8,
                g: (data >> 8) as u8,
                b: (data >> 0) as u8,
            },
            PixelFormat::Rgb => Self {
                r: (data >> 0) as u8,
                g: (data >> 8) as u8,
                b: (data >> 16) as u8,
            },
        }
    }

    pub fn to_color_code(&self, pixel_format: PixelFormat) -> u32 {
        match pixel_format {
            PixelFormat::Bgr => (self.r as u32) << 16 | (self.g as u32) << 8 | (self.b as u32) << 0,
            PixelFormat::Rgb => (self.r as u32) << 0 | (self.g as u32) << 8 | (self.b as u32) << 16,
        }
    }
}
