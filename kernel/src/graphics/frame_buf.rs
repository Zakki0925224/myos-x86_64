use super::{color::ColorCode, draw::Draw, multi_layer::Layer};
use crate::{
    arch::addr::*,
    error::Result,
    util::mutex::{Mutex, MutexError},
};
use common::graphic_info::{GraphicInfo, PixelFormat};
use core::slice;

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
        }
    }

    pub fn get_resolution(&self) -> (usize, usize) {
        self.resolution
    }

    pub fn get_stride(&self) -> usize {
        self.stride
    }

    pub fn get_format(&self) -> PixelFormat {
        self.format
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
        let frame_buf_ptr = self.frame_buf_virt_addr.as_ptr_mut::<u8>();
        let transparent_color = transparent_color.to_color_code(layer.format);

        for y in layer.y..layer.y + layer.height {
            let layer_buf_offset = (layer.width * (y - layer.y) * 4) as isize;
            let frame_buf_offset = ((res_x * y + layer.x) * 4) as isize;

            unsafe {
                // TODO: replace transparent color to frame buf color
                let buf_vec = slice::from_raw_parts(
                    layer_buf_ptr.offset(layer_buf_offset).cast::<u32>(),
                    layer.width,
                )
                .to_vec();

                // but hangged up in this code
                // for (i, data) in buf_vec.iter_mut().enumerate() {
                //     if *data == transparent_color {
                //         *data = frame_buf_ptr
                //             .offset(frame_buf_offset + (i as isize) * 4)
                //             .cast::<u32>()
                //             .read();
                //     }
                // }

                buf_vec.as_ptr().cast::<u8>().copy_to_nonoverlapping(
                    frame_buf_ptr.offset(frame_buf_offset),
                    layer.width.min(res_x - layer.x) * 4,
                );
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

        let data = *unsafe { &*(self.frame_buf_virt_addr.offset(offset).as_ptr() as *const _) };
        Ok(data)
    }

    fn write_pixel(&mut self, x: usize, y: usize, data: u32) -> Result<()> {
        let (res_x, res_y) = self.get_resolution();
        let offset = (res_x * y + x) * 4;

        if x >= res_x || y >= res_y {
            return Err(FrameBufferError::OutsideBufferAreaError { x, y }.into());
        }

        let ref_value =
            unsafe { &mut *(self.frame_buf_virt_addr.offset(offset).as_ptr_mut() as *mut _) };
        *ref_value = data;

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

pub fn get_format() -> Result<PixelFormat> {
    if let Ok(frame_buf) = unsafe { FRAME_BUF.try_lock() } {
        return Ok(frame_buf.as_ref().unwrap().get_format());
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

pub fn apply_layer_buf(layer: &Layer, transparent_color: ColorCode) -> Result<()> {
    if let Ok(mut frame_buf) = unsafe { FRAME_BUF.try_lock() } {
        return frame_buf
            .as_mut()
            .unwrap()
            .apply_layer_buf(layer, transparent_color);
    }

    Err(MutexError::Locked.into())
}
