#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    Rgb,
    Bgr,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct GraphicInfo {
    pub resolution: (u32, u32),
    pub format: PixelFormat,
    pub stride: u32,
    pub framebuf_addr: u64,
    pub framebuf_size: u64,
}
