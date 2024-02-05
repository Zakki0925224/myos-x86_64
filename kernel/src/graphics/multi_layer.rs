use super::{color::ColorCode, draw::Draw, frame_buf};
use crate::{
    error::Result,
    mem::{
        bitmap::{self, MemoryFrameInfo},
        paging::PAGE_SIZE,
    },
    util::mutex::{Mutex, MutexError},
};
use alloc::vec::Vec;
use common::graphic_info::PixelFormat;
use core::sync::atomic::{AtomicUsize, Ordering};

static mut LAYER_MAN: Mutex<Option<LayerManager>> = Mutex::new(None);

#[derive(Debug, Clone, PartialEq)]
pub enum LayerError {
    OutsideBufferAreaError { layer_id: usize, x: usize, y: usize },
    InvalidLayerIdError(usize),
    LayerManagerNotInitialized,
}

#[derive(Debug)]
pub struct Layer {
    pub id: usize,
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    buf_mem_frame_info: MemoryFrameInfo,
    pub disabled: bool,
    pub format: PixelFormat,
}

impl Drop for Layer {
    fn drop(&mut self) {
        bitmap::dealloc_mem_frame(self.buf_mem_frame_info).unwrap();
    }
}

impl Draw for Layer {
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
        for y in 0..self.height {
            for x in 0..self.width {
                self.write(x, y, color_code)?;
            }
        }

        Ok(())
    }

    fn copy(&self, x: usize, y: usize, to_x: usize, to_y: usize) -> Result<()> {
        let data = self.read(x, y)?;
        self.write(to_x, to_y, data)?;

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

impl Layer {
    pub fn new(
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        format: PixelFormat,
    ) -> Result<Self> {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let buf_mem_frame_info = bitmap::alloc_mem_frame((width * height * 4) / PAGE_SIZE + 1)?;

        Ok(Self {
            id,
            x,
            y,
            width,
            height,
            buf_mem_frame_info,
            disabled: false,
            format,
        })
    }

    pub fn move_to(&mut self, x: usize, y: usize) -> Result<()> {
        let (res_x, res_y) = frame_buf::get_resolution()?;

        if (x + self.width) > res_x || (y + self.height) > res_y {
            return Err(LayerError::OutsideBufferAreaError {
                layer_id: self.id,
                x: x + self.width,
                y: y + self.height,
            }
            .into());
        }

        self.x = x;
        self.y = y;

        Ok(())
    }

    fn read_pixel(&self, x: usize, y: usize) -> Result<u32> {
        let offset = 4 * (self.width * y) + 4 * x;

        if x >= self.width || y >= self.height {
            return Err(LayerError::OutsideBufferAreaError {
                layer_id: self.id,
                x,
                y,
            }
            .into());
        }

        let virt_addr = self.buf_mem_frame_info.get_frame_start_virt_addr();
        Ok(virt_addr.offset(offset).read_volatile())
    }

    fn write_pixel(&self, x: usize, y: usize, data: u32) -> Result<()> {
        let offset = 4 * (self.width * y) + 4 * x;

        if x >= self.width || y >= self.height {
            return Err(LayerError::OutsideBufferAreaError {
                layer_id: self.id,
                x,
                y,
            }
            .into());
        }

        let virt_addr = self.buf_mem_frame_info.get_frame_start_virt_addr();
        virt_addr.offset(offset).write_volatile(data);

        Ok(())
    }
}

struct LayerManager {
    layers: Vec<Layer>,
    pub transparent_color: ColorCode,
}

impl LayerManager {
    pub fn new(transparent_color: ColorCode) -> Self {
        Self {
            layers: Vec::new(),
            transparent_color,
        }
    }

    pub fn push_layer(&mut self, layer: Layer) {
        self.layers.push(layer);
    }

    pub fn remove_layer(&mut self, layer_id: usize) -> Result<()> {
        if self.get_layer(layer_id).is_err() {
            return Err(LayerError::InvalidLayerIdError(layer_id).into());
        }

        Ok(())
    }

    pub fn get_layer(&mut self, layer_id: usize) -> Result<&mut Layer> {
        match self.layers.iter_mut().find(|l| l.id == layer_id) {
            Some(l) => return Ok(l),
            None => return Err(LayerError::InvalidLayerIdError(layer_id).into()),
        }
    }

    pub fn draw_to_frame_buf(&self) -> Result<()> {
        for layer in &self.layers {
            if layer.disabled {
                continue;
            }

            for y in layer.y..layer.y + layer.height {
                for x in layer.x..layer.x + layer.width {
                    let color_code = layer.read(x - layer.x, y - layer.y)?;

                    if color_code == self.transparent_color {
                        continue;
                    }

                    frame_buf::write(x, y, color_code)?;
                }
            }
        }

        Ok(())
    }
}

pub fn init(transparent_color: ColorCode) -> Result<()> {
    if let Ok(mut layer_man) = unsafe { LAYER_MAN.try_lock() } {
        *layer_man = Some(LayerManager::new(transparent_color));
        return Ok(());
    }

    Err(MutexError::Locked.into())
}

pub fn transparent_color() -> Result<ColorCode> {
    if let Ok(layer_man) = unsafe { LAYER_MAN.try_lock() } {
        if let Some(layer_man) = layer_man.as_ref() {
            return Ok(layer_man.transparent_color);
        }

        return Err(LayerError::LayerManagerNotInitialized.into());
    }

    Err(MutexError::Locked.into())
}

pub fn push_layer(layer: Layer) -> Result<()> {
    if let Ok(mut layer_man) = unsafe { LAYER_MAN.try_lock() } {
        if let Some(layer_man) = layer_man.as_mut() {
            layer_man.push_layer(layer);
            return Ok(());
        }

        return Err(LayerError::LayerManagerNotInitialized.into());
    }

    Err(MutexError::Locked.into())
}

pub fn draw_to_frame_buf() -> Result<()> {
    if let Ok(layer_man) = unsafe { LAYER_MAN.try_lock() } {
        if let Some(layer_man) = layer_man.as_ref() {
            return layer_man.draw_to_frame_buf();
        }

        return Err(LayerError::LayerManagerNotInitialized.into());
    }

    Err(MutexError::Locked.into())
}

pub fn draw_layer<F: Fn(&dyn Draw) -> Result<()>>(layer_id: usize, draw: F) -> Result<()> {
    if let Ok(mut layer_man) = unsafe { LAYER_MAN.try_lock() } {
        if let Some(layer_man) = layer_man.as_mut() {
            let layer_inst = layer_man.get_layer(layer_id)?;
            return draw(layer_inst);
        }

        return Err(LayerError::LayerManagerNotInitialized.into());
    }

    Err(MutexError::Locked.into())
}
