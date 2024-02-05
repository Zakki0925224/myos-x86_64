use super::{color::ColorCode, draw::Draw};
use crate::{
    arch::addr::*,
    error::Result,
    mem::{
        bitmap::{self, MemoryFrameInfo},
        paging::PAGE_SIZE,
    },
    util::mutex::{Mutex, MutexError},
};
use common::graphic_info::{GraphicInfo, PixelFormat};

static mut FRAME_BUF: Mutex<Option<FrameBuffer>> = Mutex::new(None);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameBufferError {
    OutsideBufferAreaError { x: usize, y: usize },
}

pub struct FrameBuffer {
    resolution: (usize, usize),
    format: PixelFormat,
    frame_buf_virt_addr: VirtualAddress,
    stride: usize,
    shadow_buf_mem_frame_info: Option<MemoryFrameInfo>,
}

impl Draw for FrameBuffer {
    fn draw_rect(
        &self,
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

    fn fill(&self, color_code: ColorCode) -> Result<()> {
        let (max_x, max_y) = self.get_resolution();
        for y in 0..max_y {
            for x in 0..max_x {
                self.write(x, y, color_code)?;
            }
        }

        Ok(())
    }

    fn copy(&self, x: usize, y: usize, to_x: usize, to_y: usize) -> Result<()> {
        let data = self.read_pixel(x, y)?;
        self.write_pixel(to_x, to_y, data)?;

        Ok(())
    }

    fn read(&self, x: usize, y: usize) -> Result<ColorCode> {
        let data = self.read_pixel(x, y)?;
        Ok(ColorCode::from_pixel_data(data, self.format))
    }

    fn write(&self, x: usize, y: usize, color_code: ColorCode) -> Result<()> {
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
            shadow_buf_mem_frame_info: None,
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
        let mem_frame_info = bitmap::alloc_mem_frame(buf_size / PAGE_SIZE + 1)?;
        let virt_addr = mem_frame_info.get_frame_start_virt_addr();
        self.shadow_buf_mem_frame_info = Some(mem_frame_info);

        // copy frame buffer to shadow buffer
        let frame_buf_ptr = self.frame_buf_virt_addr.get() as *const u8;
        let shadow_buf_ptr = virt_addr.get() as *mut u8;

        unsafe {
            for _ in 0..res_y {
                frame_buf_ptr.copy_to_nonoverlapping(shadow_buf_ptr, res_x * 4);
            }
        }

        Ok(())
    }

    pub fn apply_shadow_buf(&self) {
        if self.shadow_buf_mem_frame_info.is_none() {
            return;
        }

        let (res_x, res_y) = self.get_resolution();

        // copy shadow buffer to frame buffer
        let shadow_buf_ptr = self
            .shadow_buf_mem_frame_info
            .unwrap()
            .get_frame_start_virt_addr()
            .get() as *const u8;
        let frame_buf_ptr = self.frame_buf_virt_addr.get() as *mut u8;

        // TODO: not work
        unsafe {
            for _ in 0..res_y {
                shadow_buf_ptr.copy_to_nonoverlapping(frame_buf_ptr, res_x * 4);
            }
        }
    }

    fn read_pixel(&self, x: usize, y: usize) -> Result<u32> {
        let (res_x, res_y) = self.get_resolution();
        let offset = 4 * (res_x * y) + 4 * x;

        if x >= res_x || y >= res_y {
            return Err(FrameBufferError::OutsideBufferAreaError { x, y }.into());
        }

        let virt_addr = match self.shadow_buf_mem_frame_info {
            // read from shadow buffer
            Some(info) => info.get_frame_start_virt_addr(),
            // read from frame buffer
            None => self.frame_buf_virt_addr,
        };

        Ok(virt_addr.offset(offset).read_volatile())
    }

    fn write_pixel(&self, x: usize, y: usize, data: u32) -> Result<()> {
        let (res_x, res_y) = self.get_resolution();
        let offset = 4 * (res_x * y) + 4 * x;

        if x >= res_x || y >= res_y {
            return Err(FrameBufferError::OutsideBufferAreaError { x, y }.into());
        }

        let virt_addr = match self.shadow_buf_mem_frame_info {
            // write to shadow buffer
            Some(info) => info.get_frame_start_virt_addr(),
            // write to frame buffer
            None => self.frame_buf_virt_addr,
        };

        virt_addr.offset(offset).write_volatile(data);

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
    if let Ok(frame_buf) = unsafe { FRAME_BUF.try_lock() } {
        return frame_buf
            .as_ref()
            .unwrap()
            .draw_rect(x, y, width, height, color_code);
    }

    Err(MutexError::Locked.into())
}

pub fn write(x: usize, y: usize, color_code: ColorCode) -> Result<()> {
    if let Ok(frame_buf) = unsafe { FRAME_BUF.try_lock() } {
        return frame_buf.as_ref().unwrap().write(x, y, color_code);
    }

    Err(MutexError::Locked.into())
}

pub fn copy(x: usize, y: usize, to_x: usize, to_y: usize) -> Result<()> {
    if let Ok(frame_buf) = unsafe { FRAME_BUF.try_lock() } {
        return frame_buf.as_ref().unwrap().copy(x, y, to_x, to_y);
    }

    Err(MutexError::Locked.into())
}

pub fn fill(color_code: ColorCode) -> Result<()> {
    if let Ok(frame_buf) = unsafe { FRAME_BUF.try_lock() } {
        return frame_buf.as_ref().unwrap().fill(color_code);
    }

    Err(MutexError::Locked.into())
}

pub fn apply_shadow_buf() -> Result<()> {
    if let Ok(frame_buf) = unsafe { FRAME_BUF.try_lock() } {
        frame_buf.as_ref().unwrap().apply_shadow_buf();
        return Ok(());
    }

    Err(MutexError::Locked.into())
}
