use crate::{
    addr::VirtualAddress,
    error::{Error, Result},
    fs::file::bitmap::BitmapImage,
    graphics::{
        font::FONT,
        multi_layer::{self, LayerId, LayerPositionInfo},
    },
    theme::GLOBAL_THEME,
    ColorCode,
};
use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};
use common::graphic_info::PixelFormat;

pub trait Component {
    fn layer_id_clone(&self) -> LayerId;
    fn get_layer_pos_info(&self) -> Result<LayerPositionInfo>;
    fn move_by_root(&self, to_x: usize, to_y: usize) -> Result<()>;
    fn move_by_parent(&self, parent: &dyn Component, to_x: usize, to_y: usize) -> Result<()>;
    fn draw_flush(&mut self) -> Result<()>;
}

pub struct Image {
    layer_id: LayerId,
    framebuf_virt_addr: Option<VirtualAddress>,
    pixel_format: Option<PixelFormat>,
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

    fn draw_flush(&mut self) -> Result<()> {
        if let (Some(framebuf_virt_addr), Some(pixel_format)) =
            (self.framebuf_virt_addr, self.pixel_format)
        {
            let LayerPositionInfo {
                x: _,
                y: _,
                width,
                height,
            } = multi_layer::get_layer_pos_info(&self.layer_id)?;

            for y in 0..height {
                for x in 0..width {
                    let data = unsafe {
                        framebuf_virt_addr
                            .offset((y * width + x) * 4)
                            .as_ptr::<u32>()
                            .read()
                    };
                    let color_code = ColorCode::from_pixel_data(data, pixel_format);
                    multi_layer::draw_layer(&self.layer_id, |l| l.write(x, y, color_code))?;
                }
            }
        }

        Ok(())
    }
}

impl Image {
    pub fn create_and_push_from_bitmap_image(
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
        Ok(Self {
            layer_id,
            framebuf_virt_addr: None,
            pixel_format: None,
        })
    }

    pub fn create_and_push_from_framebuf(
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        framebuf_virt_addr: VirtualAddress,
        pixel_format: PixelFormat,
    ) -> Result<Self> {
        let framebuf_virt_addr = Some(framebuf_virt_addr);
        let pixel_format = Some(pixel_format);
        let layer = multi_layer::create_layer(x, y, width, height)?;
        let layer_id = layer.id.clone();
        multi_layer::push_layer(layer)?;
        Ok(Self {
            layer_id,
            framebuf_virt_addr,
            pixel_format,
        })
    }
}

pub struct Window {
    layer_id: LayerId,
    title: String,
    close_button: Button,
    resize_button: Button,
    minimize_button: Button,
    children: Vec<Box<dyn Component>>,
    contents_base_rel_x: usize,
    contents_base_rel_y: usize,
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

    fn draw_flush(&mut self) -> Result<()> {
        let LayerPositionInfo {
            x: w_x,
            y: w_y,
            height: w_h,
            width: w_w,
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
                w_w
            } else {
                w_w - 2
            };
            let border_height = if GLOBAL_THEME.wm_component_border_flat {
                w_h
            } else {
                w_h - 2
            };

            l.draw_rect(0, 0, 2, border_height, border_color1)?;
            l.draw_rect(2, w_h - 2, w_w - 2, 2, border_color2)?;

            l.draw_rect(w_w - 2, 2, 2, w_h - 2, border_color2)?;
            l.draw_rect(0, 0, border_width, 2, border_color1)?;

            // titlebar
            l.draw_rect(
                4,
                4,
                w_w - 8,
                18,
                GLOBAL_THEME.wm_window_titlebar_back_color,
            )?;

            // title
            l.draw_string(
                7,
                7,
                &format!("<{}> {}", self.layer_id.get(), self.title),
                GLOBAL_THEME.wm_window_titlebar_fore_color,
                GLOBAL_THEME.wm_window_titlebar_back_color,
            )?;
            Ok(())
        })?;

        self.close_button.draw_flush()?;
        self.resize_button.draw_flush()?;
        self.minimize_button.draw_flush()?;

        let contents_base_rel_x = self.contents_base_rel_x;
        let mut contents_base_rel_y = self.contents_base_rel_y;
        let mut max_width = 0;

        for child in &mut self.children {
            let LayerPositionInfo {
                x: _,
                y: _,
                width,
                height,
            } = child.get_layer_pos_info()?;

            child.move_by_root(w_x + contents_base_rel_x, w_y + contents_base_rel_y)?;
            child.draw_flush()?;

            contents_base_rel_y += height + 4; // padding

            if max_width > width {
                max_width = width;
            }
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
            contents_base_rel_x: 4,
            contents_base_rel_y: 25,
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

    pub fn push_child(&mut self, child: Box<dyn Component>) -> Result<()> {
        self.children.push(child);
        Ok(())
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

    fn draw_flush(&mut self) -> Result<()> {
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
            l.draw_string(
                x,
                y,
                s,
                GLOBAL_THEME.wm_component_fore_color,
                GLOBAL_THEME.wm_component_back_color,
            )
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

    fn draw_flush(&mut self) -> Result<()> {
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
            let (font_width, font_height) = FONT.get_wh();
            l.draw_string(
                width / 2 - font_width * self.title.len() / 2,
                height / 2 - font_height / 2,
                &self.title,
                GLOBAL_THEME.wm_component_fore_color,
                GLOBAL_THEME.wm_component_back_color,
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

pub struct Label {
    layer_id: LayerId,
    label: String,
    back_color: ColorCode,
    fore_color: ColorCode,
}

impl Drop for Label {
    fn drop(&mut self) {
        let _ = multi_layer::remove_layer(&self.layer_id);
    }
}

impl Component for Label {
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

    fn draw_flush(&mut self) -> Result<()> {
        multi_layer::draw_layer(&self.layer_id, |l| {
            // back color
            l.fill(self.back_color)?;

            // label
            let (_, font_height) = FONT.get_wh();
            let c_x = 0;
            let mut c_y = 0;

            for line in self.label.lines() {
                l.draw_string(c_x, c_y, line, self.fore_color, self.back_color)?;
                c_y += font_height;
            }

            Ok(())
        })
    }
}

impl Label {
    pub fn create_and_push(
        x: usize,
        y: usize,
        label: String,
        back_color: ColorCode,
        fore_color: ColorCode,
    ) -> Result<Self> {
        // calc width and height
        let (font_width, font_height) = FONT.get_wh();
        let width = label.lines().map(|s| s.len()).max().unwrap_or(0) * font_width;
        let height = label.lines().count() * font_height;

        let layer = multi_layer::create_layer(x, y, width, height)?;
        let layer_id = layer.id.clone();
        multi_layer::push_layer(layer)?;
        Ok(Self {
            layer_id,
            label,
            back_color,
            fore_color,
        })
    }
}
