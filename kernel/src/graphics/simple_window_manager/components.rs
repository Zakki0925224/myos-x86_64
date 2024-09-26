use crate::{
    error::{Error, Result},
    fs::file::bitmap::BitmapImage,
    graphics::multi_layer::{self, LayerId, LayerPositionInfo},
    util::theme::GLOBAL_THEME,
};
use alloc::string::String;

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
            l.fill(GLOBAL_THEME.wm_window_back_color)?;
            l.draw_rect(0, 0, width, 1, GLOBAL_THEME.wm_window_border_color)?;
            l.draw_rect(0, height - 1, width, 1, GLOBAL_THEME.wm_window_border_color)?;
            l.draw_rect(0, 0, 1, height - 1, GLOBAL_THEME.wm_window_border_color)?;
            l.draw_rect(width - 1, 0, 1, height, GLOBAL_THEME.wm_window_border_color)?;
            l.draw_rect(
                1,
                1,
                width - 2,
                20,
                GLOBAL_THEME.wm_window_titlebar_back_color,
            )?; // titlebar
            l.draw_rect(
                cb_x,
                cb_y,
                cb_w,
                cb_h,
                GLOBAL_THEME.wm_window_close_button_back_color,
            )?; // close button
            l.draw_string(
                5,
                5,
                &self.title,
                GLOBAL_THEME.wm_window_titlebar_fore_color,
            )?;
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
            l.fill(GLOBAL_THEME.wm_panel_back_color)?;
            l.draw_rect(0, 0, width, 1, GLOBAL_THEME.wm_panel_border_color)?;
            l.draw_rect(0, height - 1, width, 1, GLOBAL_THEME.wm_panel_border_color)?;
            l.draw_rect(0, 0, 1, height, GLOBAL_THEME.wm_panel_border_color)?;
            l.draw_rect(width - 1, 0, 1, height, GLOBAL_THEME.wm_panel_border_color)?;
            Ok(())
        })?;
        Ok(())
    }
}
