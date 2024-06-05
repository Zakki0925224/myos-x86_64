use super::{color::RgbColorCode, draw::Draw, frame_buf};
use crate::{
    error::Result,
    fs::file::bitmap::BitmapImage,
    println,
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
    pub buf: Vec<u8>,
    pub disabled: bool,
    pub format: PixelFormat,
    pub always_on_top: bool,
}

impl Draw for Layer {
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

    fn fill(&mut self, color_code: RgbColorCode) -> Result<()> {
        for y in 0..self.height {
            for x in 0..self.width {
                self.write(x, y, color_code)?;
            }
        }

        Ok(())
    }

    fn copy(&mut self, x: usize, y: usize, to_x: usize, to_y: usize) -> Result<()> {
        let data = self.read(x, y)?;
        self.write(to_x, to_y, data)?;

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

        Ok(Self {
            id,
            x,
            y,
            width,
            height,
            buf: vec![0; width * height * 4],
            disabled: false,
            format,
            always_on_top: false,
        })
    }

    pub fn get_resolution(&self) -> (usize, usize) {
        (self.width, self.height)
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
        let offset = (self.width * y + x) * 4;

        if x >= self.width || y >= self.height {
            return Err(LayerError::OutsideBufferAreaError {
                layer_id: self.id,
                x,
                y,
            }
            .into());
        }

        let value = u32::from_le_bytes([
            self.buf[offset + 0],
            self.buf[offset + 1],
            self.buf[offset + 2],
            self.buf[offset + 3],
        ]);

        Ok(value)
    }

    fn write_pixel(&mut self, x: usize, y: usize, data: u32) -> Result<()> {
        let offset = (self.width * y + x) * 4;

        if x >= self.width || y >= self.height {
            return Err(LayerError::OutsideBufferAreaError {
                layer_id: self.id,
                x,
                y,
            }
            .into());
        }

        let [a, b, c, d] = data.to_le_bytes();
        self.buf[offset + 0] = a;
        self.buf[offset + 1] = b;
        self.buf[offset + 2] = c;
        self.buf[offset + 3] = d;
        Ok(())
    }
}

struct LayerManager {
    layers: Vec<Layer>,
    pub transparent_color: RgbColorCode,
}

impl LayerManager {
    pub fn new(transparent_color: RgbColorCode) -> Self {
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

    pub fn draw_to_frame_buf(&mut self) -> Result<()> {
        self.layers
            .sort_by(|a, b| a.always_on_top.cmp(&b.always_on_top));

        for layer in &mut self.layers {
            if layer.disabled {
                continue;
            }

            frame_buf::apply_layer_buf(layer, self.transparent_color)?;
        }

        Ok(())
    }
}

pub fn init(transparent_color: RgbColorCode) -> Result<()> {
    if let Ok(mut layer_man) = unsafe { LAYER_MAN.try_lock() } {
        *layer_man = Some(LayerManager::new(transparent_color));
        return Ok(());
    }

    Err(MutexError::Locked.into())
}

pub fn create_layer(x: usize, y: usize, width: usize, height: usize) -> Result<Layer> {
    let format = frame_buf::get_format()?;
    let layer = Layer::new(x, y, width, height, format)?;

    Ok(layer)
}

pub fn create_layer_from_bitmap_image(
    x: usize,
    y: usize,
    bitmap_image: &BitmapImage,
) -> Result<Layer> {
    let bitmap_image_info_header = bitmap_image.info_header();
    let bitmap_image_data = bitmap_image.bitmap()?;
    let width = bitmap_image_info_header.width as usize;
    let height = bitmap_image_info_header.height as usize;
    let format = frame_buf::get_format()?;
    let mut layer = Layer::new(x, y, width, height, format)?;

    for y_pos in 0..height {
        for x_pos in 0..width {
            let pixel_data = bitmap_image_data[(height - 1 - y_pos) * width + x_pos];
            layer.write_pixel(x_pos, y_pos, pixel_data.to_color_code(PixelFormat::Rgb))?;
        }
    }

    Ok(layer)
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
    if let Ok(mut layer_man) = unsafe { LAYER_MAN.try_lock() } {
        if let Some(layer_man) = layer_man.as_mut() {
            return layer_man.draw_to_frame_buf();
        }

        return Err(LayerError::LayerManagerNotInitialized.into());
    }

    Err(MutexError::Locked.into())
}

pub fn get_layer_resolution(layer_id: usize) -> Result<(usize, usize)> {
    if let Ok(mut layer_man) = unsafe { LAYER_MAN.try_lock() } {
        if let Some(layer_man) = layer_man.as_mut() {
            return Ok(layer_man.get_layer(layer_id)?.get_resolution());
        }

        return Err(LayerError::LayerManagerNotInitialized.into());
    }

    Err(MutexError::Locked.into())
}

pub fn draw_layer<F: Fn(&mut dyn Draw) -> Result<()>>(layer_id: usize, draw: F) -> Result<()> {
    if let Ok(mut layer_man) = unsafe { LAYER_MAN.try_lock() } {
        if let Some(layer_man) = layer_man.as_mut() {
            let layer_inst = layer_man.get_layer(layer_id)?;
            return draw(layer_inst);
        }

        return Err(LayerError::LayerManagerNotInitialized.into());
    }

    Err(MutexError::Locked.into())
}

pub fn move_layer(layer_id: usize, to_x: usize, to_y: usize) -> Result<()> {
    if let Ok(mut layer_man) = unsafe { LAYER_MAN.try_lock() } {
        if let Some(layer_man) = layer_man.as_mut() {
            let layer_inst = layer_man.get_layer(layer_id)?;
            layer_inst.move_to(to_x, to_y)?;
            return Ok(());
        }

        return Err(LayerError::LayerManagerNotInitialized.into());
    }

    Err(MutexError::Locked.into())
}
