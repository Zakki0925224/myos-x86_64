#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PixelFormat {
    Rgb = 0,
    Bgr = 1,
    Bgra = 2,
}

impl From<u8> for PixelFormat {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Rgb,
            1 => Self::Bgr,
            2 => Self::Bgra,
            _ => panic!("Invalid pixel format"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct GraphicInfo {
    pub resolution: (usize, usize),
    pub format: PixelFormat,
    pub stride: usize,
    pub framebuf_addr: u64,
    pub framebuf_size: usize,
}
