#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    Rgb,
    Bgr,
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
