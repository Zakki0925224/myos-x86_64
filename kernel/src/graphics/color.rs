use common::graphic_info::PixelFormat;

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
