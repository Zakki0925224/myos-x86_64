use common::graphic_info::PixelFormat;

pub const COLOR_WHITE: RGBColor = RGBColor {
    r: 255,
    g: 255,
    b: 255,
};
pub const COLOR_OLIVE: RGBColor = RGBColor {
    r: 128,
    g: 128,
    b: 0,
};
pub const COLOR_YELLOW: RGBColor = RGBColor {
    r: 255,
    g: 255,
    b: 0,
};
pub const COLOR_FUCHSIA: RGBColor = RGBColor {
    r: 255,
    g: 0,
    b: 255,
};
pub const COLOR_SILVER: RGBColor = RGBColor {
    r: 192,
    g: 192,
    b: 192,
};
pub const COLOR_CYAN: RGBColor = RGBColor {
    r: 0,
    g: 255,
    b: 255,
};
pub const COLOR_GREEN: RGBColor = RGBColor { r: 0, g: 255, b: 0 };
pub const COLOR_RED: RGBColor = RGBColor { r: 255, g: 0, b: 0 };
pub const COLOR_GRAY: RGBColor = RGBColor {
    r: 128,
    g: 128,
    b: 128,
};
pub const COLOR_BLUE: RGBColor = RGBColor { r: 0, g: 0, b: 255 };
pub const COLOR_PURPLE: RGBColor = RGBColor {
    r: 128,
    g: 0,
    b: 128,
};
pub const COLOR_BLACK: RGBColor = RGBColor { r: 0, g: 0, b: 0 };
pub const COLOR_NAVY: RGBColor = RGBColor { r: 0, g: 0, b: 128 };
pub const COLOR_TEAL: RGBColor = RGBColor {
    r: 0,
    g: 128,
    b: 128,
};
pub const COLOR_MAROON: RGBColor = RGBColor { r: 128, g: 0, b: 0 };

#[derive(Debug, Clone, Copy)]
pub struct RGBColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RGBColor {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        return RGBColor { r, g, b };
    }
}

pub trait Color {
    fn get_color_code(&self, pixel_format: PixelFormat) -> u32;
}

impl Color for RGBColor {
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
