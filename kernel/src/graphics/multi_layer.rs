use super::{
    color::RgbColorCode,
    draw::Draw,
    font::{FONT, TAB_DISP_STR},
    frame_buf,
};
use crate::{error::Result, fs::file::bitmap::BitmapImage, util::mutex::Mutex};
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

#[derive(Debug, Clone)]
pub struct LayerPositionInfo {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Clone)]
pub struct LayerId(usize);
impl LayerId {
    pub fn new() -> Self {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }

    pub fn get(&self) -> usize {
        self.0
    }
}

#[derive(Debug)]
pub struct Layer {
    pub id: LayerId,
    pub pos_info: LayerPositionInfo,
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
        let (width, height) = self.get_resolution();

        for y in 0..height {
            for x in 0..width {
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
            if char_x + font_width > self.pos_info.width {
                char_y += font_height;
                char_x = x;
            }

            if char_y + font_height > self.pos_info.height {
                continue;
            }

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
        Ok(Self {
            id: LayerId::new(),
            pos_info: LayerPositionInfo {
                x,
                y,
                width,
                height,
            },
            buf: vec![0; width * height * 4],
            disabled: false,
            format,
            always_on_top: false,
        })
    }

    pub fn move_to(&mut self, x: usize, y: usize) -> Result<()> {
        self.pos_info.x = x;
        self.pos_info.y = y;
        Ok(())
    }

    fn read_pixel(&self, x: usize, y: usize) -> Result<u32> {
        let (width, height) = self.get_resolution();
        let offset = (width * y + x) * 4;

        if x >= width || y >= height {
            return Err(LayerError::OutsideBufferAreaError {
                layer_id: self.id.get(),
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
        let (width, height) = self.get_resolution();
        let offset = (width * y + x) * 4;

        if x >= width || y >= height {
            return Err(LayerError::OutsideBufferAreaError {
                layer_id: self.id.get(),
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

    fn get_resolution(&self) -> (usize, usize) {
        let height = self.pos_info.height;
        let width = self.pos_info.width;

        (width, height)
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

    pub fn remove_layer(&mut self, layer_id: &LayerId) -> Result<()> {
        if self.get_layer(layer_id).is_err() {
            return Err(LayerError::InvalidLayerIdError(layer_id.get()).into());
        }

        self.layers.retain(|l| l.id.get() != layer_id.get());

        Ok(())
    }

    pub fn get_layer(&mut self, layer_id: &LayerId) -> Result<&mut Layer> {
        match self
            .layers
            .iter_mut()
            .find(|l| l.id.get() == layer_id.get())
        {
            Some(l) => return Ok(l),
            None => return Err(LayerError::InvalidLayerIdError(layer_id.get()).into()),
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

    pub fn get_layer_pos_info(&mut self, layer_id: &LayerId) -> Result<LayerPositionInfo> {
        let layer = self.get_layer(layer_id)?;
        Ok(layer.pos_info.clone())
    }
}

pub fn init(transparent_color: RgbColorCode) -> Result<()> {
    *unsafe { LAYER_MAN.try_lock() }? = Some(LayerManager::new(transparent_color));
    Ok(())
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
    let bitmap_image_data = bitmap_image.bitmap_to_rgb_color_code();
    let width = bitmap_image_info_header.width as usize;
    let height = bitmap_image_info_header.height as usize;
    let format = frame_buf::get_format()?;
    let mut layer = Layer::new(x, y, width, height, format)?;

    for h in 0..height {
        for w in 0..width {
            let pixel_data = bitmap_image_data[h * width + w];
            layer.write_pixel(w, h, pixel_data.to_color_code(PixelFormat::Bgr))?;
        }
    }

    Ok(layer)
}

pub fn push_layer(layer: Layer) -> Result<()> {
    unsafe { LAYER_MAN.try_lock() }?
        .as_mut()
        .ok_or(LayerError::LayerManagerNotInitialized)?
        .push_layer(layer);
    Ok(())
}

pub fn draw_to_frame_buf() -> Result<()> {
    unsafe { LAYER_MAN.try_lock() }?
        .as_mut()
        .ok_or(LayerError::LayerManagerNotInitialized)?
        .draw_to_frame_buf()
}

pub fn draw_layer<F: Fn(&mut dyn Draw) -> Result<()>>(layer_id: &LayerId, draw: F) -> Result<()> {
    draw(
        unsafe { LAYER_MAN.try_lock() }?
            .as_mut()
            .ok_or(LayerError::LayerManagerNotInitialized)?
            .get_layer(layer_id)?,
    )
}

pub fn move_layer(layer_id: &LayerId, to_x: usize, to_y: usize) -> Result<()> {
    unsafe { LAYER_MAN.try_lock() }?
        .as_mut()
        .ok_or(LayerError::LayerManagerNotInitialized)?
        .get_layer(layer_id)?
        .move_to(to_x, to_y)
}

pub fn remove_layer(layer_id: &LayerId) -> Result<()> {
    unsafe { LAYER_MAN.try_lock() }?
        .as_mut()
        .ok_or(LayerError::LayerManagerNotInitialized)?
        .remove_layer(layer_id)
}

pub fn get_layer_pos_info(layer_id: &LayerId) -> Result<LayerPositionInfo> {
    unsafe { LAYER_MAN.try_lock() }?
        .as_mut()
        .ok_or(LayerError::LayerManagerNotInitialized)?
        .get_layer_pos_info(layer_id)
}
