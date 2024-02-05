use common::graphic_info::PixelFormat;

pub const COLOR_WHITE: ColorCode = ColorCode::new(255, 255, 255);
pub const COLOR_OLIVE: ColorCode = ColorCode::new(128, 128, 0);
pub const COLOR_YELLOW: ColorCode = ColorCode::new(255, 255, 0);
pub const COLOR_FUCHSIA: ColorCode = ColorCode::new(255, 0, 255);
pub const COLOR_SILVER: ColorCode = ColorCode::new(192, 192, 192);
pub const COLOR_CYAN: ColorCode = ColorCode::new(0, 255, 255);
pub const COLOR_GREEN: ColorCode = ColorCode::new(0, 255, 0);
pub const COLOR_RED: ColorCode = ColorCode::new(255, 0, 0);
pub const COLOR_GRAY: ColorCode = ColorCode::new(128, 128, 128);
pub const COLOR_BLUE: ColorCode = ColorCode::new(0, 0, 255);
pub const COLOR_PURPLE: ColorCode = ColorCode::new(128, 0, 128);
pub const COLOR_BLACK: ColorCode = ColorCode::new(0, 0, 0);
pub const COLOR_NAVY: ColorCode = ColorCode::new(0, 0, 128);
pub const COLOR_TEAL: ColorCode = ColorCode::new(0, 128, 128);
pub const COLOR_MAROON: ColorCode = ColorCode::new(128, 0, 0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorCode {
    Rgb { r: u8, g: u8, b: u8 },
}

impl From<(u8, u8, u8)> for ColorCode {
    fn from(color: (u8, u8, u8)) -> Self {
        Self::Rgb {
            r: color.0,
            g: color.1,
            b: color.2,
        }
    }
}

impl ColorCode {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        ColorCode::Rgb { r, g, b }
    }

    pub fn from_pixel_data(data: u32, pixel_format: PixelFormat) -> Self {
        match pixel_format {
            PixelFormat::Bgr => Self::Rgb {
                r: (data >> 16) as u8,
                g: (data >> 8) as u8,
                b: (data >> 0) as u8,
            },
            PixelFormat::Rgb => Self::Rgb {
                r: (data >> 0) as u8,
                g: (data >> 8) as u8,
                b: (data >> 16) as u8,
            },
        }
    }

    pub fn to_color_code(&self, pixel_format: PixelFormat) -> u32 {
        match self {
            Self::Rgb { r, g, b } => match pixel_format {
                PixelFormat::Bgr => (*r as u32) << 16 | (*g as u32) << 8 | (*b as u32) << 0,
                PixelFormat::Rgb => (*r as u32) << 0 | (*g as u32) << 8 | (*b as u32) << 16,
            },
        }
    }
}
