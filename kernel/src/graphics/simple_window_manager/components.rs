use crate::{
    error::{Error, Result},
    fs::file::bitmap::BitmapImage,
    graphics::{
        font::FONT,
        multi_layer::{self, LayerId, LayerPositionInfo},
    },
    theme::GLOBAL_THEME,
};
use alloc::string::String;

pub trait Component {
    fn layer_id_clone(&self) -> LayerId;
    fn get_layer_pos_info(&self) -> Result<LayerPositionInfo>;
    fn move_by_root(&self, to_x: usize, to_y: usize) -> Result<()>;
    fn draw_fresh(&mut self) -> Result<()>;
}

pub struct Image {
    layer_id: LayerId,
}

impl Drop for Image {
    fn drop(&mut self) {
        let _ = multi_layer::remove_layer(&self.layer_id);
    }
}

impl Component for Image {
    fn layer_id_clone(&self) -> LayerId {
        self.layer_id.clone()
    }

    fn get_layer_pos_info(&self) -> Result<LayerPositionInfo> {
        multi_layer::get_layer_pos_info(&self.layer_id)
    }

    fn move_by_root(&self, to_x: usize, to_y: usize) -> Result<()> {
        multi_layer::move_layer(&self.layer_id, to_x, to_y)
    }

    fn draw_fresh(&mut self) -> Result<()> {
        Ok(())
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
    layer_id: LayerId,
    title: String,
    close_button_pos: (usize, usize),
    close_button_size: (usize, usize),
    pub is_closed: bool,
}

impl Drop for Window {
    fn drop(&mut self) {
        let _ = multi_layer::remove_layer(&self.layer_id);
    }
}

impl Component for Window {
    fn layer_id_clone(&self) -> LayerId {
        self.layer_id.clone()
    }

    fn get_layer_pos_info(&self) -> Result<LayerPositionInfo> {
        multi_layer::get_layer_pos_info(&self.layer_id)
    }

    fn move_by_root(&self, to_x: usize, to_y: usize) -> Result<()> {
        multi_layer::move_layer(&self.layer_id, to_x, to_y)
    }

    fn draw_fresh(&mut self) -> Result<()> {
        let (cb_x, cb_y) = self.close_button_pos;
        let (cb_w, cb_h) = self.close_button_size;

        let LayerPositionInfo {
            x: _,
            y: _,
            width,
            height,
        } = self.get_layer_pos_info()?;
        multi_layer::draw_layer(&self.layer_id, |l| {
            // back color
            l.fill(GLOBAL_THEME.wm_panel_back_color)?;

            // borders
            l.draw_rect(0, 0, 2, height - 2, GLOBAL_THEME.wm_panel_border_color1)?;
            l.draw_rect(
                2,
                height - 2,
                width - 2,
                2,
                GLOBAL_THEME.wm_panel_border_color2,
            )?;

            l.draw_rect(
                width - 2,
                2,
                2,
                height - 2,
                GLOBAL_THEME.wm_panel_border_color2,
            )?;
            l.draw_rect(0, 0, width - 2, 2, GLOBAL_THEME.wm_panel_border_color1)?;

            // titlebar
            l.draw_rect(
                4,
                4,
                width - 8,
                18,
                GLOBAL_THEME.wm_window_titlebar_back_color,
            )?;

            // title
            l.draw_string(
                7,
                7,
                &format!("<{}> {}", self.layer_id.get(), self.title),
                GLOBAL_THEME.wm_window_titlebar_fore_color,
            )?;
            Ok(())
        })?;

        // close button
        self.draw_button(cb_x, cb_y, cb_w, cb_h, "x")?;

        // resize button
        self.draw_button(cb_x - cb_w - 2, cb_y, cb_w, cb_h, "[]")?;

        // minimize button
        self.draw_button(cb_x - cb_w * 2 - 4, cb_y, cb_w, cb_h, "-")
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
            close_button_pos: (width - 8 - 14, 6),
            close_button_size: (16, 14),
            is_closed: false,
        })
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn is_close_button_clickable(&self, x: usize, y: usize) -> Result<bool> {
        let (cb_x, cb_y) = self.close_button_pos;
        let (cb_w, cb_h) = self.close_button_size;
        let LayerPositionInfo {
            x: w_x,
            y: w_y,
            width: _,
            height: _,
        } = self.get_layer_pos_info()?;

        Ok(x >= w_x + cb_x && x < w_x + cb_x + cb_w && y >= w_y + cb_y && y < w_y + cb_y + cb_h)
    }

    fn draw_button(
        &self,
        b_x: usize,
        b_y: usize,
        b_w: usize,
        b_h: usize,
        title: &str,
    ) -> Result<()> {
        multi_layer::draw_layer(&self.layer_id, |l| {
            l.draw_rect(b_x, b_y, b_w, b_h, GLOBAL_THEME.wm_panel_back_color)?;
            l.draw_rect(b_x, b_y, 2, b_h - 2, GLOBAL_THEME.wm_panel_border_color1)?;
            l.draw_rect(
                b_x + 2,
                b_y + b_h - 2,
                b_w - 2,
                2,
                GLOBAL_THEME.wm_panel_border_color2,
            )?;

            l.draw_rect(
                b_x + b_w - 2,
                b_y + 2,
                2,
                b_h - 2,
                GLOBAL_THEME.wm_panel_border_color2,
            )?;
            l.draw_rect(b_x, b_y, b_w - 2, 2, GLOBAL_THEME.wm_panel_border_color1)?;

            let (f_w, f_h) = (FONT.get_width(), FONT.get_height());
            l.draw_string(
                b_x + b_w / 2 - f_w * title.len() / 2,
                b_y + b_h / 2 - f_h / 2,
                title,
                GLOBAL_THEME.wm_panel_fore_color,
            )?;

            Ok(())
        })
    }
}

pub struct Panel {
    layer_id: LayerId,
}

impl Drop for Panel {
    fn drop(&mut self) {
        let _ = multi_layer::remove_layer(&self.layer_id);
    }
}

impl Component for Panel {
    fn layer_id_clone(&self) -> LayerId {
        self.layer_id.clone()
    }

    fn get_layer_pos_info(&self) -> Result<LayerPositionInfo> {
        multi_layer::get_layer_pos_info(&self.layer_id)
    }

    fn move_by_root(&self, to_x: usize, to_y: usize) -> Result<()> {
        multi_layer::move_layer(&self.layer_id, to_x, to_y)
    }

    fn draw_fresh(&mut self) -> Result<()> {
        let LayerPositionInfo {
            x: _,
            y: _,
            width,
            height,
        } = self.get_layer_pos_info()?;

        multi_layer::draw_layer(&self.layer_id, |l| {
            // back color
            l.fill(GLOBAL_THEME.wm_panel_back_color)?;

            // borders
            // borders
            l.draw_rect(0, 0, 2, height - 2, GLOBAL_THEME.wm_panel_border_color1)?;
            l.draw_rect(
                2,
                height - 2,
                width - 2,
                2,
                GLOBAL_THEME.wm_panel_border_color2,
            )?;

            l.draw_rect(
                width - 2,
                2,
                2,
                height - 2,
                GLOBAL_THEME.wm_panel_border_color2,
            )?;
            l.draw_rect(0, 0, width - 2, 2, GLOBAL_THEME.wm_panel_border_color1)?;

            Ok(())
        })
    }
}

impl Panel {
    pub fn create_and_push(x: usize, y: usize, width: usize, height: usize) -> Result<Self> {
        let layer = multi_layer::create_layer(x, y, width, height)?;
        let layer_id = layer.id.clone();
        multi_layer::push_layer(layer)?;
        Ok(Self { layer_id })
    }

    pub fn draw_string(&self, x: usize, y: usize, s: &str) -> Result<()> {
        multi_layer::draw_layer(&self.layer_id, |l| {
            l.draw_string(x, y, s, GLOBAL_THEME.wm_panel_fore_color)
        })
    }
}
