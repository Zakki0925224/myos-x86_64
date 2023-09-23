use common::graphic_info::{GraphicInfo, PixelFormat};
use lazy_static::lazy_static;
use spin::Mutex;

use crate::arch::addr::*;

use super::color::Color;

lazy_static! {
    pub static ref FRAME_BUF: Mutex<FrameBuffer> = Mutex::new(FrameBuffer::new());
}

#[derive(Debug)]
pub enum FrameBufferError {
    NotInitialized,
    OutsideFrameBufferAreaError(usize, usize), // x, y
}

pub struct FrameBuffer {
    is_init: bool,
    resolution: (usize, usize),
    format: PixelFormat,
    framebuf_virt_addr: VirtualAddress,
    framebuf_size: usize,
    stride: usize,
}

impl FrameBuffer {
    pub fn new() -> Self {
        return Self {
            is_init: false,
            resolution: (0, 0),
            format: PixelFormat::Rgb,
            framebuf_virt_addr: VirtualAddress::default(),
            framebuf_size: 0,
            stride: 0,
        };
    }

    pub fn init(&mut self, graphic_info: GraphicInfo) {
        self.resolution = (
            graphic_info.resolution.0 as usize,
            graphic_info.resolution.1 as usize,
        );
        self.format = graphic_info.format;
        self.framebuf_virt_addr = VirtualAddress::new(graphic_info.framebuf_addr);
        self.framebuf_size = graphic_info.framebuf_size as usize;
        self.stride = graphic_info.stride as usize;
        self.is_init = true;
    }

    pub fn is_init(&self) -> bool {
        return self.is_init;
    }

    pub fn get_resolution(&self) -> (usize, usize) {
        return self.resolution;
    }

    pub fn get_stride(&self) -> usize {
        return self.stride;
    }

    pub fn get_pixel_format(&self) -> PixelFormat {
        return self.format;
    }

    pub fn draw_rect(
        &self,
        x1: usize,
        y1: usize,
        width: usize,
        height: usize,
        color: &impl Color,
    ) -> Result<(), FrameBufferError> {
        if !self.is_init {
            return Err(FrameBufferError::NotInitialized);
        }

        let (res_x, res_y) = self.get_resolution();
        if x1 >= res_x || y1 >= res_y {
            return Err(FrameBufferError::OutsideFrameBufferAreaError(x1, y1));
        }

        if x1 + width >= res_x || y1 + height >= res_y {
            return Err(FrameBufferError::OutsideFrameBufferAreaError(
                x1 + width,
                y1 + height,
            ));
        }

        for y in y1..y1 + height {
            for x in x1..x1 + width {
                self.set_color(x, y, color);
            }
        }

        return Ok(());
    }

    pub fn copy_pixel(
        &self,
        x: usize,
        y: usize,
        to_x: usize,
        to_y: usize,
    ) -> Result<(), FrameBufferError> {
        if !self.is_init {
            return Err(FrameBufferError::NotInitialized);
        }

        let (res_x, res_y) = self.get_resolution();
        if x >= res_x || y >= res_y {
            return Err(FrameBufferError::OutsideFrameBufferAreaError(x, y));
        }

        if to_x >= res_x || to_y >= res_y {
            return Err(FrameBufferError::OutsideFrameBufferAreaError(to_x, to_y));
        }

        let data = self.read_pixel(x, y);
        self.write_pixel(to_x, to_y, data);

        return Ok(());
    }

    pub fn clear(&self, color: &impl Color) -> Result<(), FrameBufferError> {
        if !self.is_init {
            return Err(FrameBufferError::NotInitialized);
        }

        let (max_x, max_y) = self.get_resolution();
        for y in 0..max_y {
            for x in 0..max_x {
                self.set_color(x, y, color);
            }
        }

        return Ok(());
    }

    fn set_color(&self, x: usize, y: usize, color: &impl Color) {
        self.write_pixel(x, y, color.get_color_code(self.get_pixel_format()));
    }

    fn read_pixel(&self, x: usize, y: usize) -> u32 {
        let (res_x, _) = self.get_resolution();
        return self
            .framebuf_virt_addr
            .offset(4 * (res_x * y) + 4 * x)
            .read_volatile();
    }

    fn write_pixel(&self, x: usize, y: usize, data: u32) {
        let (res_x, _) = self.get_resolution();
        self.framebuf_virt_addr
            .offset(4 * (res_x * y) + 4 * x)
            .write_volatile(data);
    }
}
