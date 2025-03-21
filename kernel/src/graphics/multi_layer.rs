use super::{
    color::ColorCode,
    draw::Draw,
    font::{FONT, TAB_DISP_STR},
    frame_buf,
};
use crate::{
    error::{Error, Result},
    fs::file::bitmap::BitmapImage,
    util::mutex::Mutex,
};
use alloc::vec::Vec;
use common::graphic_info::PixelFormat;
use core::sync::atomic::{AtomicUsize, Ordering};

static mut LAYER_MAN: Mutex<LayerManager> = Mutex::new(LayerManager::new());

#[derive(Debug, Clone, PartialEq)]
pub enum LayerError {
    OutsideBufferAreaError { layer_id: usize, x: usize, y: usize },
    InvalidLayerIdError(usize),
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

    pub const fn new_val(value: i64) -> Result<Self> {
        if value < 0 {
            return Err(Error::Failed("Invalid layer id"));
        }

        Ok(Self(value as usize))
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
        let (width, height) = self.get_resolution();

        for y in 0..height {
            for x in 0..width {
                self.write(x, y, color_code)?;
            }
        }

        Ok(())
    }

    fn draw_string(
        &mut self,
        x: usize,
        y: usize,
        s: &str,
        fore_color: ColorCode,
        back_color: ColorCode,
    ) -> Result<()> {
        let (font_width, font_height) = FONT.get_wh();
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
                        self.draw_font(char_x, char_y, c, fore_color, back_color)?;
                        char_x += font_width;
                    }
                }
                _ => (),
            }

            self.draw_font(char_x, char_y, c, fore_color, back_color)?;
            char_x += font_width;
        }

        Ok(())
    }

    fn draw_font(
        &mut self,
        x: usize,
        y: usize,
        c: char,
        fore_color: ColorCode,
        back_color: ColorCode,
    ) -> Result<()> {
        let (font_width, font_height) = FONT.get_wh();
        let glyph = FONT.get_glyph(c)?;

        for h in 0..font_height {
            for w in 0..font_width {
                let color = if (glyph[h] << w) & 0x80 == 0x80 {
                    fore_color
                } else {
                    back_color
                };
                self.draw_rect(x + w, y + h, 1, 1, color)?;
            }
        }

        Ok(())
    }

    fn copy(&mut self, x: usize, y: usize, to_x: usize, to_y: usize) -> Result<()> {
        let data = self.read(x, y)?;
        self.write(to_x, to_y, data)?;

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
}

impl LayerManager {
    const fn new() -> Self {
        Self { layers: Vec::new() }
    }

    fn push_layer(&mut self, layer: Layer) {
        self.layers.push(layer);
    }

    fn remove_layer(&mut self, layer_id: &LayerId) -> Result<()> {
        if self.get_layer(layer_id).is_err() {
            return Err(LayerError::InvalidLayerIdError(layer_id.get()).into());
        }

        self.layers.retain(|l| l.id.get() != layer_id.get());

        Ok(())
    }

    fn get_layer(&mut self, layer_id: &LayerId) -> Result<&mut Layer> {
        self.layers
            .iter_mut()
            .find(|l| l.id.get() == layer_id.get())
            .ok_or(LayerError::InvalidLayerIdError(layer_id.get()).into())
    }

    fn draw_to_frame_buf(&mut self) -> Result<()> {
        self.layers
            .sort_by(|a, b| a.always_on_top.cmp(&b.always_on_top));

        for layer in &mut self.layers {
            if layer.disabled {
                continue;
            }

            frame_buf::apply_layer_buf(layer)?;
        }

        Ok(())
    }

    fn get_layer_pos_info(&mut self, layer_id: &LayerId) -> Result<LayerPositionInfo> {
        let layer = self.get_layer(layer_id)?;
        Ok(layer.pos_info.clone())
    }
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
    unsafe { LAYER_MAN.try_lock() }?.push_layer(layer);
    Ok(())
}

pub fn draw_to_frame_buf() -> Result<()> {
    unsafe { LAYER_MAN.try_lock() }?.draw_to_frame_buf()
}

pub fn draw_layer<F: Fn(&mut dyn Draw) -> Result<()>>(layer_id: &LayerId, draw: F) -> Result<()> {
    draw(unsafe { LAYER_MAN.try_lock() }?.get_layer(layer_id)?)
}

pub fn move_layer(layer_id: &LayerId, to_x: usize, to_y: usize) -> Result<()> {
    unsafe { LAYER_MAN.try_lock() }?
        .get_layer(layer_id)?
        .move_to(to_x, to_y)
}

pub fn remove_layer(layer_id: &LayerId) -> Result<()> {
    unsafe { LAYER_MAN.try_lock() }?.remove_layer(layer_id)
}

pub fn get_layer_pos_info(layer_id: &LayerId) -> Result<LayerPositionInfo> {
    unsafe { LAYER_MAN.try_lock() }?.get_layer_pos_info(layer_id)
}
