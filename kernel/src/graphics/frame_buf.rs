use super::{
    color::RgbColorCode,
    draw::Draw,
    font::{FONT, TAB_DISP_STR},
    multi_layer::{Layer, LayerPositionInfo},
};
use crate::{arch::addr::*, error::Result, util::mutex::Mutex};
use alloc::vec::Vec;
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
    NotInitialized,
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
        color_code: RgbColorCode,
    ) -> Result<()> {
        for y in y..y + height {
            for x in x..x + width {
                self.write(x, y, color_code)?;
            }
        }

        Ok(())
    }

    fn draw_string(&mut self, x: usize, y: usize, s: &str, color_code: RgbColorCode) -> Result<()> {
        let font_width = FONT.get_width();
        let font_height = FONT.get_height();
        let mut char_x = x;
        let mut char_y = y;

        for c in s.chars() {
            match c {
                '\n' => {
                    char_y += font_height;
                    continue;
                }
                '\t' => {
                    for c in TAB_DISP_STR.chars() {
                        self.draw_font(char_x, char_y, c, color_code)?;
                        char_x += font_width;
                    }
                }
                _ => (),
            }

            self.draw_font(char_x, char_y, c, color_code)?;
            char_x += font_width;
        }

        Ok(())
    }

    fn draw_font(&mut self, x: usize, y: usize, c: char, color_code: RgbColorCode) -> Result<()> {
        let glyph = FONT.get_glyph(FONT.unicode_char_to_glyph_index(c))?;

        for h in 0..FONT.get_height() {
            for w in 0..FONT.get_width() {
                if !(glyph[h] << w) & 0x80 == 0x80 {
                    continue;
                }

                self.draw_rect(x + w, y + h, 1, 1, color_code)?;
            }
        }

        Ok(())
    }

    fn fill(&mut self, color_code: RgbColorCode) -> Result<()> {
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

    fn read(&self, x: usize, y: usize) -> Result<RgbColorCode> {
        let data = self.read_pixel(x, y)?;
        Ok(RgbColorCode::from_pixel_data(data, self.format))
    }

    fn write(&mut self, x: usize, y: usize, color_code: RgbColorCode) -> Result<()> {
        self.write_pixel(x, y, color_code.to_color_code(self.format))
    }
}

impl FrameBuffer {
    pub fn new(graphic_info: &GraphicInfo) -> Self {
        Self {
            resolution: graphic_info.resolution,
            format: graphic_info.format,
            frame_buf_virt_addr: graphic_info.framebuf_addr.into(),
            stride: graphic_info.stride,
            shadow_buf: None,
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

    pub fn enable_shadow_buf(&mut self) {
        let (res_x, res_y) = self.resolution;
        let len = res_x * res_y * 4;
        let mut shadow_buf = vec![0; len];

        // copy data from frame buf
        let shadow_buf_ptr = shadow_buf.as_mut_ptr();
        let frame_buf_ptr = self.frame_buf_virt_addr.as_ptr();

        unsafe {
            shadow_buf_ptr.copy_from_nonoverlapping(frame_buf_ptr, len);
        }

        self.shadow_buf = Some(shadow_buf);
    }

    // no check disabled layer
    pub fn apply_layer_buf(
        &mut self,
        layer: &mut Layer,
        transparent_color: RgbColorCode,
    ) -> Result<()> {
        if layer.format != self.format {
            return Err(FrameBufferError::InvalidPixelFormatError {
                _self: self.format,
                target: layer.format,
            }
            .into());
        }

        let (res_x, res_y) = self.get_resolution();
        let layer_buf_ptr = layer.buf.as_mut_ptr();
        let frame_buf_ptr = if let Some(shadow_buf) = &mut self.shadow_buf {
            shadow_buf.as_mut_ptr()
        } else {
            self.frame_buf_virt_addr.as_ptr_mut()
        };

        let transparent_color = transparent_color.to_color_code(layer.format);
        let LayerPositionInfo {
            x: layer_x,
            y: layer_y,
            width: layer_width,
            height: layer_height,
        } = layer.pos_info;

        let layer_x = layer_x.min(res_x);
        let layer_y = layer_y.min(res_y);
        let layer_y_end = (layer_y + layer_height).min(res_y);

        for y in layer_y..layer_y_end {
            let layer_buf_offset = (layer_width * (y - layer_y) * 4) as isize;
            let frame_buf_offset = ((res_x * y + layer_x) * 4) as isize;

            unsafe {
                let buf = slice::from_raw_parts_mut(
                    layer_buf_ptr.offset(layer_buf_offset).cast::<u32>(),
                    layer_width,
                );

                // TODO: replace transparent color to frame buf color
                //but hangged up in this code
                // for (i, data) in buf.iter_mut().enumerate() {
                //     if *data == transparent_color {
                //         *data = frame_buf_ptr
                //             .offset(frame_buf_offset + (i as isize) * 4)
                //             .cast::<u32>()
                //             .read();
                //     }
                // }

                frame_buf_ptr
                    .offset(frame_buf_offset)
                    .copy_from_nonoverlapping(
                        buf.as_ptr().cast::<u8>(),
                        layer_width.min(res_x - layer_x) * 4,
                    );
            }
        }

        Ok(())
    }

    pub fn apply_shadow_buf(&self) {
        if self.shadow_buf.is_none() {
            return;
        }

        let (res_x, res_y) = self.resolution;
        let len = res_x * res_y * 4;
        let shadow_buf_ptr = self.shadow_buf.as_ref().unwrap().as_ptr();
        self.frame_buf_virt_addr
            .copy_from_nonoverlapping(shadow_buf_ptr, len);
    }

    fn read_pixel(&self, x: usize, y: usize) -> Result<u32> {
        let (res_x, res_y) = self.get_resolution();
        let offset = (res_x * y + x) * 4;

        if x >= res_x || y >= res_y {
            return Err(FrameBufferError::OutsideBufferAreaError { x, y }.into());
        }

        let data = unsafe {
            if let Some(shadow_buf) = &self.shadow_buf {
                *shadow_buf.as_ptr().add(offset).cast()
            } else {
                *&*(self.frame_buf_virt_addr.offset(offset).as_ptr() as *const _)
            }
        };

        Ok(data)
    }

    fn write_pixel(&mut self, x: usize, y: usize, data: u32) -> Result<()> {
        let (res_x, res_y) = self.get_resolution();
        let offset = (res_x * y + x) * 4;

        if x >= res_x || y >= res_y {
            return Err(FrameBufferError::OutsideBufferAreaError { x, y }.into());
        }

        unsafe {
            let ref_value = if let Some(shadow_buf) = &mut self.shadow_buf {
                shadow_buf.as_mut_ptr().add(offset).cast()
            } else {
                &mut *(self.frame_buf_virt_addr.offset(offset).as_ptr_mut() as *mut _)
            };
            *ref_value = data;
        }

        Ok(())
    }
}

pub fn init(graphic_info: &GraphicInfo) -> Result<()> {
    *unsafe { FRAME_BUF.get_force_mut() } = Some(FrameBuffer::new(graphic_info));
    Ok(())
}

pub fn get_resolution() -> Result<(usize, usize)> {
    Ok(unsafe { FRAME_BUF.try_lock() }?
        .as_ref()
        .ok_or(FrameBufferError::NotInitialized)?
        .get_resolution())
}

pub fn get_stride() -> Result<usize> {
    Ok(unsafe { FRAME_BUF.try_lock() }?
        .as_ref()
        .ok_or(FrameBufferError::NotInitialized)?
        .get_stride())
}

pub fn get_format() -> Result<PixelFormat> {
    Ok(unsafe { FRAME_BUF.try_lock() }?
        .as_ref()
        .ok_or(FrameBufferError::NotInitialized)?
        .get_format())
}

pub fn draw_rect(
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    color_code: RgbColorCode,
) -> Result<()> {
    unsafe { FRAME_BUF.try_lock() }?
        .as_mut()
        .ok_or(FrameBufferError::NotInitialized)?
        .draw_rect(x, y, width, height, color_code)
}

pub fn draw_font(x: usize, y: usize, c: char, color_code: RgbColorCode) -> Result<()> {
    unsafe { FRAME_BUF.try_lock() }?
        .as_mut()
        .ok_or(FrameBufferError::NotInitialized)?
        .draw_font(x, y, c, color_code)
}

pub fn copy(x: usize, y: usize, to_x: usize, to_y: usize) -> Result<()> {
    unsafe { FRAME_BUF.try_lock() }?
        .as_mut()
        .ok_or(FrameBufferError::NotInitialized)?
        .copy(x, y, to_x, to_y)
}

pub fn fill(color_code: RgbColorCode) -> Result<()> {
    unsafe { FRAME_BUF.try_lock() }?
        .as_mut()
        .ok_or(FrameBufferError::NotInitialized)?
        .fill(color_code)
}

pub fn enable_shadow_buf() -> Result<()> {
    unsafe { FRAME_BUF.try_lock() }?
        .as_mut()
        .ok_or(FrameBufferError::NotInitialized)?
        .enable_shadow_buf();
    Ok(())
}

pub fn apply_layer_buf(layer: &mut Layer, transparent_color: RgbColorCode) -> Result<()> {
    unsafe { FRAME_BUF.try_lock() }?
        .as_mut()
        .ok_or(FrameBufferError::NotInitialized)?
        .apply_layer_buf(layer, transparent_color)
}

pub fn apply_shadow_buf() -> Result<()> {
    unsafe { FRAME_BUF.try_lock() }?
        .as_mut()
        .ok_or(FrameBufferError::NotInitialized)?
        .apply_shadow_buf();
    Ok(())
}
