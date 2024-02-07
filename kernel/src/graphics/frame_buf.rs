use super::{color::ColorCode, draw::Draw, multi_layer::Layer};
use crate::{
    arch::addr::*,
    error::Result,
    util::mutex::{Mutex, MutexError},
};
use alloc::vec::Vec;
use common::graphic_info::{GraphicInfo, PixelFormat};

static mut FRAME_BUF: Mutex<Option<FrameBuffer>> = Mutex::new(None);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameBufferError {
    OutsideBufferAreaError {
        x: usize,
        y: usize,
    },
    InvalidPixelFormatError {
        _self: PixelFormat,
        target: PixelFormat,
    },
}

pub struct FrameBuffer {
    resolution: (usize, usize),
    format: PixelFormat,
    frame_buf_virt_addr: VirtualAddress,
    stride: usize,
    shadow_buf: Option<Vec<u8>>,
}

impl Draw for FrameBuffer {
    fn draw_rect(
        &mut self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        color_code: ColorCode,
    ) -> Result<()> {
        for y in y..y + height {
            for x in x..x + width {
                self.write(x, y, color_code)?;
            }
        }

        Ok(())
    }

    fn fill(&mut self, color_code: ColorCode) -> Result<()> {
        let (max_x, max_y) = self.get_resolution();
        for y in 0..max_y {
            for x in 0..max_x {
                self.write(x, y, color_code)?;
            }
        }

        Ok(())
    }

    fn copy(&mut self, x: usize, y: usize, to_x: usize, to_y: usize) -> Result<()> {
        let data = self.read_pixel(x, y)?;
        self.write_pixel(to_x, to_y, data)?;

        Ok(())
    }

    fn read(&self, x: usize, y: usize) -> Result<ColorCode> {
        let data = self.read_pixel(x, y)?;
        Ok(ColorCode::from_pixel_data(data, self.format))
    }

    fn write(&mut self, x: usize, y: usize, color_code: ColorCode) -> Result<()> {
        self.write_pixel(x, y, color_code.to_color_code(self.format))
    }
}

impl FrameBuffer {
    pub fn new(graphic_info: GraphicInfo) -> Self {
        let resolution = (
            graphic_info.resolution.0 as usize,
            graphic_info.resolution.1 as usize,
        );
        let format = graphic_info.format;
        let frame_buf_virt_addr = VirtualAddress::new(graphic_info.framebuf_addr);
        let stride = graphic_info.stride as usize;

        Self {
            resolution,
            format,
            frame_buf_virt_addr,
            stride,
            shadow_buf: None,
        }
    }

    pub fn get_resolution(&self) -> (usize, usize) {
        self.resolution
    }

    pub fn get_stride(&self) -> usize {
        self.stride
    }

    pub fn enable_shadow_buf(&mut self) -> Result<()> {
        let (res_x, res_y) = self.get_resolution();
        let buf_size = res_x * res_y * 4;
        self.shadow_buf = Some(vec![0; buf_size]);

        // copy frame buffer to shadow buffer
        let shadow_buf_ptr = self.shadow_buf.as_mut().unwrap().as_mut_ptr();
        let frame_buf_ptr = self.frame_buf_virt_addr.get() as *const u8;

        for y in 0..res_y {
            let offset = res_x * y * 4;
            unsafe {
                frame_buf_ptr
                    .add(offset)
                    .copy_to_nonoverlapping(shadow_buf_ptr.add(offset), res_x * 4);
            }
        }

        Ok(())
    }

    pub fn apply_shadow_buf(&self) {
        if self.shadow_buf.is_none() {
            return;
        }

        let (res_x, res_y) = self.get_resolution();

        let shadow_buf_ptr = self.shadow_buf.as_ref().unwrap().as_ptr();
        let frame_buf_ptr = self.frame_buf_virt_addr.get() as *mut u8;

        for y in 0..res_y {
            let offset = res_x * y * 4;
            unsafe {
                shadow_buf_ptr
                    .add(offset)
                    .copy_to_nonoverlapping(frame_buf_ptr.add(offset), res_x * 4);
            }
        }
    }

    // no check disabled layer
    pub fn apply_layer_buf(&mut self, layer: &Layer, transparent_color: ColorCode) -> Result<()> {
        if layer.format != self.format {
            return Err(FrameBufferError::InvalidPixelFormatError {
                _self: self.format,
                target: layer.format,
            }
            .into());
        }

        let (res_x, _) = self.get_resolution();
        let layer_buf_ptr = layer.buf.as_ptr();
        let frame_buf_ptr = match self.shadow_buf.as_mut() {
            Some(buf) => buf.as_mut_ptr(),
            None => self.frame_buf_virt_addr.get() as *mut u8,
        };

        for y in layer.y..layer.y + layer.height {
            for x in layer.x..layer.x + layer.width {
                let color_code = layer.read(x - layer.x, y - layer.y)?;

                if color_code == transparent_color {
                    continue;
                }

                let layer_buf_offset = (layer.width * (y - layer.y) + (x - layer.x)) * 4;
                let frame_buf_offset = (res_x * y + x) * 4;
                unsafe {
                    layer_buf_ptr
                        .add(layer_buf_offset)
                        .copy_to_nonoverlapping(frame_buf_ptr.add(frame_buf_offset), 4);
                }
            }
        }

        Ok(())
    }

    fn read_pixel(&self, x: usize, y: usize) -> Result<u32> {
        let (res_x, res_y) = self.get_resolution();
        let offset = (res_x * y + x) * 4;

        if x >= res_x || y >= res_y {
            return Err(FrameBufferError::OutsideBufferAreaError { x, y }.into());
        }

        let data = match self.shadow_buf.as_ref() {
            Some(buf) => u32::from_le_bytes([
                buf[offset + 0],
                buf[offset + 1],
                buf[offset + 2],
                buf[offset + 3],
            ]),
            None => self.frame_buf_virt_addr.offset(offset).read_volatile(),
        };

        Ok(data)
    }

    fn write_pixel(&mut self, x: usize, y: usize, data: u32) -> Result<()> {
        let (res_x, res_y) = self.get_resolution();
        let offset = (res_x * y + x) * 4;

        if x >= res_x || y >= res_y {
            return Err(FrameBufferError::OutsideBufferAreaError { x, y }.into());
        }

        match self.shadow_buf.as_mut() {
            Some(buf) => {
                let [a, b, c, d] = data.to_le_bytes();
                buf[offset + 0] = a;
                buf[offset + 1] = b;
                buf[offset + 2] = c;
                buf[offset + 3] = d;
            }
            None => self.frame_buf_virt_addr.offset(offset).write_volatile(data),
        }

        Ok(())
    }
}

pub fn init(graphic_info: GraphicInfo) -> Result<()> {
    if let Ok(mut frame_buf) = unsafe { FRAME_BUF.try_lock() } {
        *frame_buf = Some(FrameBuffer::new(graphic_info));
        return Ok(());
    }

    Err(MutexError::Locked.into())
}

pub fn get_resolution() -> Result<(usize, usize)> {
    if let Ok(frame_buf) = unsafe { FRAME_BUF.try_lock() } {
        return Ok(frame_buf.as_ref().unwrap().get_resolution());
    }

    Err(MutexError::Locked.into())
}

pub fn get_stride() -> Result<usize> {
    if let Ok(frame_buf) = unsafe { FRAME_BUF.try_lock() } {
        return Ok(frame_buf.as_ref().unwrap().get_stride());
    }

    Err(MutexError::Locked.into())
}

pub fn enable_shadow_buf() -> Result<()> {
    if let Ok(mut frame_buf) = unsafe { FRAME_BUF.try_lock() } {
        return frame_buf.as_mut().unwrap().enable_shadow_buf();
    }

    Err(MutexError::Locked.into())
}

pub fn draw_rect(
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    color_code: ColorCode,
) -> Result<()> {
    if let Ok(mut frame_buf) = unsafe { FRAME_BUF.try_lock() } {
        return frame_buf
            .as_mut()
            .unwrap()
            .draw_rect(x, y, width, height, color_code);
    }

    Err(MutexError::Locked.into())
}

pub fn copy(x: usize, y: usize, to_x: usize, to_y: usize) -> Result<()> {
    if let Ok(mut frame_buf) = unsafe { FRAME_BUF.try_lock() } {
        return frame_buf.as_mut().unwrap().copy(x, y, to_x, to_y);
    }

    Err(MutexError::Locked.into())
}

pub fn fill(color_code: ColorCode) -> Result<()> {
    if let Ok(mut frame_buf) = unsafe { FRAME_BUF.try_lock() } {
        return frame_buf.as_mut().unwrap().fill(color_code);
    }

    Err(MutexError::Locked.into())
}

pub fn apply_shadow_buf() -> Result<()> {
    if let Ok(mut frame_buf) = unsafe { FRAME_BUF.try_lock() } {
        frame_buf.as_mut().unwrap().apply_shadow_buf();
        return Ok(());
    }

    Err(MutexError::Locked.into())
}

pub fn apply_layer_buf(layer: &Layer, transparent_color: ColorCode) -> Result<()> {
    if let Ok(mut frame_buf) = unsafe { FRAME_BUF.try_lock() } {
        return frame_buf
            .as_mut()
            .unwrap()
            .apply_layer_buf(layer, transparent_color);
    }

    Err(MutexError::Locked.into())
}
