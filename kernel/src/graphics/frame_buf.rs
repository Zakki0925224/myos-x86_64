use super::{color::ColorCode, draw::Draw, multi_layer::Layer};
use crate::{
    arch::addr::VirtualAddress,
    error::{Error, Result},
    util::mutex::Mutex,
};
use alloc::vec::Vec;
use common::graphic_info::{GraphicInfo, PixelFormat};

static mut FB: Mutex<FrameBuffer> = Mutex::new(FrameBuffer::new());

struct FrameBuffer {
    resolution: Option<(usize, usize)>,
    stride: Option<usize>,
    format: Option<PixelFormat>,
    frame_buf_virt_addr: Option<VirtualAddress>,
    shadow_buf: Option<Vec<u32>>,
}

impl Draw for FrameBuffer {
    fn resolution(&self) -> Result<(usize, usize)> {
        let res = self.resolution.ok_or_else(|| Error::NotInitialized)?;
        let stride = self.stride.ok_or_else(|| Error::NotInitialized)?;
        Ok((stride, res.1))
    }

    fn format(&self) -> Result<PixelFormat> {
        self.format.ok_or_else(|| Error::NotInitialized)
    }

    fn buf_ptr(&self) -> Result<*const u32> {
        if let Some(shadow_buf) = &self.shadow_buf {
            Ok(shadow_buf.as_ptr())
        } else {
            let addr = self
                .frame_buf_virt_addr
                .ok_or_else(|| Error::NotInitialized)?;
            Ok(addr.as_ptr())
        }
    }

    fn buf_ptr_mut(&mut self) -> Result<*mut u32> {
        if let Some(shadow_buf) = &mut self.shadow_buf {
            Ok(shadow_buf.as_mut_ptr())
        } else {
            let addr = self
                .frame_buf_virt_addr
                .ok_or_else(|| Error::NotInitialized)?;
            Ok(addr.as_ptr_mut())
        }
    }
}

impl FrameBuffer {
    const fn new() -> Self {
        Self {
            resolution: None,
            stride: None,
            format: None,
            frame_buf_virt_addr: None,
            shadow_buf: None,
        }
    }

    fn init(&mut self, graphic_info: &GraphicInfo) -> Result<()> {
        self.resolution = Some(graphic_info.resolution);
        self.stride = Some(graphic_info.stride);
        self.format = Some(graphic_info.format);
        self.frame_buf_virt_addr = Some(graphic_info.framebuf_addr.into());

        Ok(())
    }

    fn enable_shadow_buf(&mut self) -> Result<()> {
        let (res_x, res_y) = self.resolution()?;
        let buf = Vec::with_capacity(res_x * res_y);
        self.shadow_buf = Some(buf);

        // copy the current framebuffer to shadow buffer
        let buf_ptr: *mut u32 = self
            .frame_buf_virt_addr
            .ok_or_else(|| Error::NotInitialized)?
            .as_ptr_mut();
        let shadow_buf_ptr = self.buf_ptr_mut()?;

        unsafe {
            buf_ptr.copy_to(shadow_buf_ptr, res_x * res_y);
        }

        Ok(())
    }

    fn apply_shadow_buf(&mut self) -> Result<()> {
        if self.shadow_buf.is_none() {
            return Ok(());
        }

        let (res_x, res_y) = self.resolution()?;
        let buf_ptr: *mut u32 = self
            .frame_buf_virt_addr
            .ok_or_else(|| Error::NotInitialized)?
            .as_ptr_mut();
        let shadow_buf_ptr = self.buf_ptr_mut()?;

        unsafe {
            shadow_buf_ptr.copy_to(buf_ptr, res_x * res_y);
        }

        Ok(())
    }

    fn apply_layer_buf(&mut self, layer: &Layer) -> Result<()> {
        let layer_xy = layer.layer_pos_info().xy;
        layer.copy_to(self, layer_xy)
    }
}

pub fn init(graphic_info: &GraphicInfo) -> Result<()> {
    let mut fb = unsafe { FB.try_lock() }?;
    fb.init(graphic_info)?;
    Ok(())
}

pub fn resolution() -> Result<(usize, usize)> {
    let fb = unsafe { FB.try_lock() }?;
    fb.resolution()
}

pub fn format() -> Result<PixelFormat> {
    let fb = unsafe { FB.try_lock() }?;
    fb.format()
}

pub fn fill(color: ColorCode) -> Result<()> {
    let mut fb = unsafe { FB.try_lock() }?;
    fb.fill(color)
}

pub fn draw_rect(xy: (usize, usize), wh: (usize, usize), color: ColorCode) -> Result<()> {
    let mut fb = unsafe { FB.try_lock() }?;
    fb.draw_rect(xy, wh, color)
}

pub fn copy_rect(src_xy: (usize, usize), dst_xy: (usize, usize), wh: (usize, usize)) -> Result<()> {
    let mut fb = unsafe { FB.try_lock() }?;
    fb.copy_rect(src_xy, dst_xy, wh)
}

pub fn draw_char(
    xy: (usize, usize),
    c: char,
    fore_color: ColorCode,
    back_color: ColorCode,
) -> Result<()> {
    let mut fb = unsafe { FB.try_lock() }?;
    fb.draw_char(xy, c, fore_color, back_color)
}

pub fn enable_shadow_buf() -> Result<()> {
    let mut fb = unsafe { FB.try_lock() }?;
    fb.enable_shadow_buf()
}

pub fn apply_shadow_buf() -> Result<()> {
    let mut fb = unsafe { FB.try_lock() }?;
    fb.apply_shadow_buf()
}

pub fn apply_layer_buf(layer: &Layer) -> Result<()> {
    let mut fb = unsafe { FB.try_lock() }?;
    fb.apply_layer_buf(layer)
}
