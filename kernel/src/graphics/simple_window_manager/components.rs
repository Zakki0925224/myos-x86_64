use crate::{
    error::{Error, Result},
    fs::file::bitmap::BitmapImage,
    graphics::{
        font::FONT,
        multi_layer::{self, LayerId, LayerPositionInfo},
    },
    theme::GLOBAL_THEME,
};
use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};

pub trait Component {
    fn layer_id_clone(&self) -> LayerId;
    fn get_layer_pos_info(&self) -> Result<LayerPositionInfo>;
    fn move_by_root(&self, to_x: usize, to_y: usize) -> Result<()>;
    fn move_by_parent(&self, parent: &dyn Component, to_x: usize, to_y: usize) -> Result<()>;
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

    fn move_by_parent(&self, parent: &dyn Component, to_x: usize, to_y: usize) -> Result<()> {
        let LayerPositionInfo {
            x: p_x,
            y: p_y,
            width: _,
            height: _,
        } = parent.get_layer_pos_info()?;

        let LayerPositionInfo {
            x,
            y,
            width: _,
            height: _,
        } = self.get_layer_pos_info()?;

        self.move_by_root(to_x + x - p_x, to_y + y - p_y)
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

pub struct Window {
    layer_id: LayerId,
    title: String,
    close_button: Button,
    resize_button: Button,
    minimize_button: Button,
    children: Vec<Box<dyn Component>>,
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
        self.close_button.move_by_parent(self, to_x, to_y)?;
        self.resize_button.move_by_parent(self, to_x, to_y)?;
        self.minimize_button.move_by_parent(self, to_x, to_y)?;

        for child in &self.children {
            child.move_by_parent(self, to_x, to_y)?;
        }

        multi_layer::move_layer(&self.layer_id, to_x, to_y)?;

        Ok(())
    }

    fn move_by_parent(&self, parent: &dyn Component, to_x: usize, to_y: usize) -> Result<()> {
        let LayerPositionInfo {
            x: p_x,
            y: p_y,
            width: _,
            height: _,
        } = parent.get_layer_pos_info()?;

        let LayerPositionInfo {
            x,
            y,
            width: _,
            height: _,
        } = self.get_layer_pos_info()?;

        self.move_by_root(to_x + x - p_x, to_y + y - p_y)
    }

    fn draw_fresh(&mut self) -> Result<()> {
        let LayerPositionInfo {
            x: _,
            y: _,
            height,
            width,
        } = self.get_layer_pos_info()?;
        multi_layer::draw_layer(&self.layer_id, |l| {
            // back color
            l.fill(GLOBAL_THEME.wm_component_back_color)?;

            // borders
            let border_color1 = GLOBAL_THEME.wm_component_border_color1;
            let border_color2 = if GLOBAL_THEME.wm_component_border_flat {
                GLOBAL_THEME.wm_component_border_color1
            } else {
                GLOBAL_THEME.wm_component_border_color2
            };
            let border_width = if GLOBAL_THEME.wm_component_border_flat {
                width
            } else {
                width - 2
            };
            let border_height = if GLOBAL_THEME.wm_component_border_flat {
                height
            } else {
                height - 2
            };

            l.draw_rect(0, 0, 2, border_height, border_color1)?;
            l.draw_rect(2, height - 2, width - 2, 2, border_color2)?;

            l.draw_rect(width - 2, 2, 2, height - 2, border_color2)?;
            l.draw_rect(0, 0, border_width, 2, border_color1)?;

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

        self.close_button.draw_fresh()?;
        self.resize_button.draw_fresh()?;
        self.minimize_button.draw_fresh()?;

        for child in &mut self.children {
            child.draw_fresh()?;
        }

        Ok(())
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

        let close_button =
            Button::create_and_push("x".to_string(), x + width - 8 - 14, y + 6, 16, 14)?;
        let resize_button =
            Button::create_and_push("[]".to_string(), x + width - 8 - 14 - 16 - 2, y + 6, 16, 14)?;
        let minimize_button =
            Button::create_and_push("_".to_string(), x + width - 8 - 14 - 32 - 4, y + 6, 16, 14)?;

        Ok(Self {
            layer_id,
            title,
            is_closed: false,
            close_button,
            resize_button,
            children: Vec::new(),
            minimize_button,
        })
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn is_close_button_clickable(&self, x: usize, y: usize) -> Result<bool> {
        let LayerPositionInfo {
            x: cb_x,
            y: cb_y,
            width: cb_w,
            height: cb_h,
        } = self.close_button.get_layer_pos_info()?;

        Ok(x >= cb_x && x < cb_x + cb_w && y >= cb_y && y < cb_y + cb_h)
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

    fn move_by_parent(&self, parent: &dyn Component, to_x: usize, to_y: usize) -> Result<()> {
        let LayerPositionInfo {
            x: p_x,
            y: p_y,
            width: _,
            height: _,
        } = parent.get_layer_pos_info()?;

        let LayerPositionInfo {
            x,
            y,
            width: _,
            height: _,
        } = self.get_layer_pos_info()?;

        self.move_by_root(to_x + x - p_x, to_y + y - p_y)
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
            l.fill(GLOBAL_THEME.wm_component_back_color)?;

            // borders
            let border_color1 = GLOBAL_THEME.wm_component_border_color1;
            let border_color2 = if GLOBAL_THEME.wm_component_border_flat {
                GLOBAL_THEME.wm_component_border_color1
            } else {
                GLOBAL_THEME.wm_component_border_color2
            };
            let border_width = if GLOBAL_THEME.wm_component_border_flat {
                width
            } else {
                width - 2
            };
            let border_height = if GLOBAL_THEME.wm_component_border_flat {
                height
            } else {
                height - 2
            };

            l.draw_rect(0, 0, 2, border_height, border_color1)?;
            l.draw_rect(2, height - 2, width - 2, 2, border_color2)?;

            l.draw_rect(width - 2, 2, 2, height - 2, border_color2)?;
            l.draw_rect(0, 0, border_width, 2, border_color1)?;

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
            l.draw_string(x, y, s, GLOBAL_THEME.wm_component_fore_color)
        })
    }
}

pub struct Button {
    layer_id: LayerId,
    title: String,
}

impl Drop for Button {
    fn drop(&mut self) {
        let _ = multi_layer::remove_layer(&self.layer_id);
    }
}

impl Component for Button {
    fn layer_id_clone(&self) -> LayerId {
        self.layer_id.clone()
    }

    fn get_layer_pos_info(&self) -> Result<LayerPositionInfo> {
        multi_layer::get_layer_pos_info(&self.layer_id)
    }

    fn move_by_root(&self, to_x: usize, to_y: usize) -> Result<()> {
        multi_layer::move_layer(&self.layer_id, to_x, to_y)
    }

    fn move_by_parent(&self, parent: &dyn Component, to_x: usize, to_y: usize) -> Result<()> {
        let LayerPositionInfo {
            x: p_x,
            y: p_y,
            width: _,
            height: _,
        } = parent.get_layer_pos_info()?;

        let LayerPositionInfo {
            x,
            y,
            width: _,
            height: _,
        } = self.get_layer_pos_info()?;

        self.move_by_root(to_x + x - p_x, to_y + y - p_y)
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
            l.fill(GLOBAL_THEME.wm_component_back_color)?;

            // borders
            let border_color1 = GLOBAL_THEME.wm_component_border_color1;
            let border_color2 = if GLOBAL_THEME.wm_component_border_flat {
                GLOBAL_THEME.wm_component_border_color1
            } else {
                GLOBAL_THEME.wm_component_border_color2
            };
            let border_width = if GLOBAL_THEME.wm_component_border_flat {
                width
            } else {
                width - 2
            };
            let border_height = if GLOBAL_THEME.wm_component_border_flat {
                height
            } else {
                height - 2
            };

            l.draw_rect(0, 0, 2, border_height, border_color1)?;
            l.draw_rect(2, height - 2, width - 2, 2, border_color2)?;

            l.draw_rect(width - 2, 2, 2, height - 2, border_color2)?;
            l.draw_rect(0, 0, border_width, 2, border_color1)?;

            // title
            let (f_w, f_h) = (FONT.get_width(), FONT.get_height());
            l.draw_string(
                width / 2 - f_w * self.title.len() / 2,
                height / 2 - f_h / 2,
                &self.title,
                GLOBAL_THEME.wm_component_fore_color,
            )?;

            Ok(())
        })
    }
}

impl Button {
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
        Ok(Self { layer_id, title })
    }
}
