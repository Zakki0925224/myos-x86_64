use super::color::Color;
use crate::{
    arch::addr::*,
    error::Result,
    util::mutex::{Mutex, MutexError},
};
use common::graphic_info::{GraphicInfo, PixelFormat};

static mut FRAME_BUF: Mutex<Option<FrameBuffer>> = Mutex::new(None);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameBufferError {
    OutsideFrameBufferAreaError { x: usize, y: usize },
}

pub struct FrameBuffer {
    resolution: (usize, usize),
    format: PixelFormat,
    framebuf_virt_addr: VirtualAddress,
    stride: usize,
}

impl FrameBuffer {
    pub fn new(graphic_info: GraphicInfo) -> Self {
        let resolution = (
            graphic_info.resolution.0 as usize,
            graphic_info.resolution.1 as usize,
        );
        let format = graphic_info.format;
        let framebuf_virt_addr = VirtualAddress::new(graphic_info.framebuf_addr);
        let stride = graphic_info.stride as usize;

        Self {
            resolution,
            format,
            framebuf_virt_addr,
            stride,
        }
    }

    pub fn get_resolution(&self) -> (usize, usize) {
        self.resolution
    }

    pub fn get_stride(&self) -> usize {
        self.stride
    }

    pub fn get_pixel_format(&self) -> PixelFormat {
        self.format
    }

    pub fn draw_rect<C: Color>(
        &self,
        x1: usize,
        y1: usize,
        width: usize,
        height: usize,
        color: &C,
    ) -> Result<()> {
        let (res_x, res_y) = self.get_resolution();
        if x1 >= res_x || y1 >= res_y {
            return Err(FrameBufferError::OutsideFrameBufferAreaError { x: x1, y: y1 }.into());
        }

        if x1 + width >= res_x || y1 + height >= res_y {
            return Err(FrameBufferError::OutsideFrameBufferAreaError {
                x: x1 + width,
                y: y1 + height,
            }
            .into());
        }

        for y in y1..y1 + height {
            for x in x1..x1 + width {
                self.set_color(x, y, color);
            }
        }

        Ok(())
    }

    pub fn copy_pixel(&self, x: usize, y: usize, to_x: usize, to_y: usize) -> Result<()> {
        let (res_x, res_y) = self.get_resolution();
        if x >= res_x || y >= res_y {
            return Err(FrameBufferError::OutsideFrameBufferAreaError { x, y }.into());
        }

        if to_x >= res_x || to_y >= res_y {
            return Err(FrameBufferError::OutsideFrameBufferAreaError { x: to_x, y: to_y }.into());
        }

        let data = self.read_pixel(x, y);
        self.write_pixel(to_x, to_y, data);

        Ok(())
    }

    pub fn clear<C: Color>(&self, color: &C) {
        let (max_x, max_y) = self.get_resolution();
        for y in 0..max_y {
            for x in 0..max_x {
                self.set_color(x, y, color);
            }
        }
    }

    fn set_color<C: Color>(&self, x: usize, y: usize, color: &C) {
        self.write_pixel(x, y, color.get_color_code(self.get_pixel_format()));
    }

    fn read_pixel(&self, x: usize, y: usize) -> u32 {
        let (res_x, _) = self.get_resolution();
        self.framebuf_virt_addr
            .offset(4 * (res_x * y) + 4 * x)
            .read_volatile()
    }

    fn write_pixel(&self, x: usize, y: usize, data: u32) {
        let (res_x, _) = self.get_resolution();

        // read_volatile is slow
        // if self.read_pixel(x, y) == data {
        //     return;
        // }

        self.framebuf_virt_addr
            .offset(4 * (res_x * y) + 4 * x)
            .write_volatile(data);
    }
}

pub fn init(graphic_info: GraphicInfo) -> Result<()> {
    if let Ok(mut frame_buf) = unsafe { FRAME_BUF.try_lock() } {
        *frame_buf = Some(FrameBuffer::new(graphic_info));
        return Ok(());
    }

    Err(MutexError::Locked.into())
}

pub fn get_resolution() -> (usize, usize) {
    if let Ok(frame_buf) = unsafe { FRAME_BUF.try_lock() } {
        return frame_buf.as_ref().unwrap().get_resolution();
    }

    (0, 0)
}

pub fn get_stride() -> usize {
    if let Ok(frame_buf) = unsafe { FRAME_BUF.try_lock() } {
        return frame_buf.as_ref().unwrap().get_stride();
    }

    0
}

pub fn get_pixel_format() -> PixelFormat {
    if let Ok(frame_buf) = unsafe { FRAME_BUF.try_lock() } {
        return frame_buf.as_ref().unwrap().get_pixel_format();
    }

    PixelFormat::Rgb
}

pub fn draw_rect<C: Color>(
    x1: usize,
    y1: usize,
    width: usize,
    height: usize,
    color: &C,
) -> Result<()> {
    if let Ok(frame_buf) = unsafe { FRAME_BUF.try_lock() } {
        return frame_buf
            .as_ref()
            .unwrap()
            .draw_rect(x1, y1, width, height, color);
    }

    Err(MutexError::Locked.into())
}

pub fn copy_pixel(x: usize, y: usize, to_x: usize, to_y: usize) -> Result<()> {
    if let Ok(frame_buf) = unsafe { FRAME_BUF.try_lock() } {
        return frame_buf.as_ref().unwrap().copy_pixel(x, y, to_x, to_y);
    }

    Err(MutexError::Locked.into())
}

pub fn clear<C: Color>(color: &C) -> Result<()> {
    if let Ok(frame_buf) = unsafe { FRAME_BUF.try_lock() } {
        frame_buf.as_ref().unwrap().clear(color);
        return Ok(());
    }

    Err(MutexError::Locked.into())
}
