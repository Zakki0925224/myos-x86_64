use common::graphic_info::PixelFormat;

pub const COLOR_WHITE: RgbColorCode = RgbColorCode::new(255, 255, 255);
// pub const COLOR_OLIVE: RgbColorCode = RgbColorCode::new(128, 128, 0);
// pub const COLOR_YELLOW: RgbColorCode = RgbColorCode::new(255, 255, 0);
// pub const COLOR_FUCHSIA: RgbColorCode = RgbColorCode::new(255, 0, 255);
// pub const COLOR_SILVER: RgbColorCode = RgbColorCode::new(192, 192, 192);
// pub const COLOR_CYAN: RgbColorCode = RgbColorCode::new(0, 255, 255);
// pub const COLOR_GREEN: RgbColorCode = RgbColorCode::new(0, 255, 0);
// pub const COLOR_RED: RgbColorCode = RgbColorCode::new(255, 0, 0);
// pub const COLOR_GRAY: RgbColorCode = RgbColorCode::new(128, 128, 128);
// pub const COLOR_BLUE: RgbColorCode = RgbColorCode::new(0, 0, 255);
// pub const COLOR_PURPLE: RgbColorCode = RgbColorCode::new(128, 0, 128);
pub const COLOR_BLACK: RgbColorCode = RgbColorCode::new(0, 0, 0);
// pub const COLOR_NAVY: RgbColorCode = RgbColorCode::new(0, 0, 128);
// pub const COLOR_TEAL: RgbColorCode = RgbColorCode::new(0, 128, 128);
// pub const COLOR_MAROON: RgbColorCode = RgbColorCode::new(128, 0, 0);

// nord colors: https://www.nordtheme.com/
pub const PN_COLOR_1: RgbColorCode = RgbColorCode::new(0x2e, 0x34, 0x40);
pub const PN_COLOR_2: RgbColorCode = RgbColorCode::new(0x3b, 0x42, 0x52);
pub const PN_COLOR_3: RgbColorCode = RgbColorCode::new(0x43, 0x4c, 0x5e);
pub const PN_COLOR_4: RgbColorCode = RgbColorCode::new(0x4c, 0x56, 0x6a);
pub const SS_COLOR_1: RgbColorCode = RgbColorCode::new(0xd8, 0xde, 0xe9);
pub const SS_COLOR_2: RgbColorCode = RgbColorCode::new(0xe5, 0xe9, 0xf0);
pub const SS_COLOR_3: RgbColorCode = RgbColorCode::new(0xec, 0xef, 0xf4);
pub const FR_COLOR_1: RgbColorCode = RgbColorCode::new(0x8f, 0xbc, 0xbb);
pub const FR_COLOR_2: RgbColorCode = RgbColorCode::new(0x88, 0xc0, 0xd0);
pub const FR_COLOR_3: RgbColorCode = RgbColorCode::new(0x81, 0xa1, 0xc1);
pub const FR_COLOR_4: RgbColorCode = RgbColorCode::new(0x5e, 0x81, 0xac);
pub const AU_COLOR_1: RgbColorCode = RgbColorCode::new(0xbf, 0x61, 0x6a); // red
pub const AU_COLOR_2: RgbColorCode = RgbColorCode::new(0xd0, 0x87, 0x70); // orange
pub const AU_COLOR_3: RgbColorCode = RgbColorCode::new(0xeb, 0xcb, 0x8b); // yellow
pub const AU_COLOR_4: RgbColorCode = RgbColorCode::new(0xa3, 0xbe, 0x8c); // green
pub const AU_COLOR_5: RgbColorCode = RgbColorCode::new(0xb4, 0x8e, 0xad); // purple

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
