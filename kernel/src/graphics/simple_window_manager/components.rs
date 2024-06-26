use super::*;
use crate::{
    error::{Error, Result},
    fs::file::bitmap::BitmapImage,
    graphics::multi_layer::{self, LayerId},
};
use alloc::string::String;

// nord colors: https://www.nordtheme.com/
const PN_COLOR_1: RgbColorCode = RgbColorCode::new(0x2e, 0x34, 0x40);
const PN_COLOR_2: RgbColorCode = RgbColorCode::new(0x3b, 0x42, 0x52);
const PN_COLOR_3: RgbColorCode = RgbColorCode::new(0x43, 0x4c, 0x5e);
const PN_COLOR_4: RgbColorCode = RgbColorCode::new(0x4c, 0x56, 0x6a);
const SS_COLOR_1: RgbColorCode = RgbColorCode::new(0xd8, 0xde, 0xe9);
const SS_COLOR_2: RgbColorCode = RgbColorCode::new(0xe5, 0xe9, 0xf0);
const SS_COLOR_3: RgbColorCode = RgbColorCode::new(0xec, 0xef, 0xf4);
const FR_COLOR_1: RgbColorCode = RgbColorCode::new(0x8f, 0xbc, 0xbb);
const FR_COLOR_2: RgbColorCode = RgbColorCode::new(0x88, 0xc0, 0xd0);
const FR_COLOR_3: RgbColorCode = RgbColorCode::new(0x81, 0xa1, 0xc1);
const FR_COLOR_4: RgbColorCode = RgbColorCode::new(0x5e, 0x81, 0xac);
const AU_COLOR_1: RgbColorCode = RgbColorCode::new(0xbf, 0x61, 0x6a); // red
const AU_COLOR_2: RgbColorCode = RgbColorCode::new(0xd0, 0x87, 0x70); // orange
const AU_COLOR_3: RgbColorCode = RgbColorCode::new(0xeb, 0xcb, 0x8b); // yellow
const AU_COLOR_4: RgbColorCode = RgbColorCode::new(0xa3, 0xbe, 0x8c); // green
const AU_COLOR_5: RgbColorCode = RgbColorCode::new(0xb4, 0x8e, 0xad); // purple

pub struct Image {
    pub layer_id: LayerId,
}

impl Drop for Image {
    fn drop(&mut self) {
        let _ = multi_layer::remove_layer(&self.layer_id);
    }
}

impl Image {
    pub fn create_and_push(
        bitmap_image: &BitmapImage,
        x: usize,
        y: usize,
        always_on_top: bool,
    ) -> Result<Self> {
        if !bitmap_image.is_valid() {
            return Err(Error::Failed("Invalid bitmap image"));
        }

        let mut layer = multi_layer::create_layer_from_bitmap_image(x, y, bitmap_image)?;
        layer.always_on_top = always_on_top;
        let layer_id = layer.id.clone();
        multi_layer::push_layer(layer)?;
        Ok(Self { layer_id })
    }
}

#[derive(Debug)]
pub struct Window {
    pub layer_id: LayerId,
    pub title: String,
    pub close_button_pos: (usize, usize),
    pub close_button_size: (usize, usize),
    pub is_closed: bool,
}

impl Drop for Window {
    fn drop(&mut self) {
        let _ = multi_layer::remove_layer(&self.layer_id);
    }
}

impl Window {
    pub fn create_and_push(
        title: String,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> Result<Self> {
        let layer = multi_layer::create_layer(x, y, width, height)?;
        let layer_id = layer.id.clone();
        multi_layer::push_layer(layer)?;
        Ok(Self {
            layer_id,
            title,
            close_button_pos: (width - 1 - 20, 1),
            close_button_size: (20, 20),
            is_closed: false,
        })
    }

    pub fn draw_fresh(&self) -> Result<()> {
        let (cb_x, cb_y) = self.close_button_pos;
        let (cb_w, cb_h) = self.close_button_size;

        let LayerPositionInfo {
            x: _,
            y: _,
            width,
            height,
        } = multi_layer::get_layer_pos_info(&self.layer_id)?;
        multi_layer::draw_layer(&self.layer_id, |l| {
            l.fill(PN_COLOR_1)?;
            l.draw_rect(0, 0, width, 1, PN_COLOR_4)?;
            l.draw_rect(0, height - 1, width, 1, PN_COLOR_4)?;
            l.draw_rect(0, 0, 1, height - 1, PN_COLOR_4)?;
            l.draw_rect(width - 1, 0, 1, height, PN_COLOR_4)?;
            l.draw_rect(1, 1, width - 2, 20, PN_COLOR_2)?; // titlebar
            l.draw_rect(cb_x, cb_y, cb_w, cb_h, AU_COLOR_1)?; // close button
            l.draw_string(5, 5, &self.title, SS_COLOR_1)?;
            Ok(())
        })?;
        Ok(())
    }
}

pub struct Panel {
    pub layer_id: LayerId,
}

impl Drop for Panel {
    fn drop(&mut self) {
        let _ = multi_layer::remove_layer(&self.layer_id);
    }
}

impl Panel {
    pub fn create_and_push(x: usize, y: usize, width: usize, height: usize) -> Result<Self> {
        let layer = multi_layer::create_layer(x, y, width, height)?;
        let layer_id = layer.id.clone();
        multi_layer::push_layer(layer)?;
        Ok(Self { layer_id })
    }

    pub fn draw_fresh(&self) -> Result<()> {
        let LayerPositionInfo {
            x: _,
            y: _,
            width,
            height,
        } = multi_layer::get_layer_pos_info(&self.layer_id)?;
        multi_layer::draw_layer(&self.layer_id, |l| {
            l.fill(PN_COLOR_2)?;
            l.draw_rect(0, 0, width, 1, PN_COLOR_3)?;
            l.draw_rect(0, height - 1, width, 1, PN_COLOR_3)?;
            l.draw_rect(0, 0, 1, height, PN_COLOR_3)?;
            l.draw_rect(width - 1, 0, 1, height, PN_COLOR_3)?;
            Ok(())
        })?;
        Ok(())
    }
}
