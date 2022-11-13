#[derive(Debug, Copy, Clone)]
pub enum PixelFormat
{
    Rgb,
    Bgr,
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct GraphicInfo
{
    pub resolution: (u32, u32),
    pub format: PixelFormat,
    pub stride: usize,
    pub framebuf_addr: u64,
    pub framebuf_size: u64,
}
