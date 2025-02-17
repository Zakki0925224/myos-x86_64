use super::{
    font::{FONT, TAB_DISP_STR},
    frame_buf,
    multi_layer::{self, LayerId, LayerPositionInfo},
};
use crate::{error::Result, theme::GLOBAL_THEME, util::mutex::Mutex, ColorCode};
use core::fmt::{self, Write};

static mut FRAME_BUF_CONSOLE: Mutex<Option<FrameBufferConsole>> = Mutex::new(None);

pub struct FrameBufferConsole {
    back_color: ColorCode,
    default_fore_color: ColorCode,
    fore_color: ColorCode,
    max_x_res: usize,
    max_y_res: usize,
    char_max_x_len: usize,
    char_max_y_len: usize,
    cursor_x: usize,
    cursor_y: usize,
    target_layer_id: Option<LayerId>,
    is_scrollable: bool,
}

impl FrameBufferConsole {
    pub fn new(back_color: ColorCode, fore_color: ColorCode, is_scrollable: bool) -> Result<Self> {
        let max_x_res = frame_buf::get_stride()?;
        let max_y_res = frame_buf::get_resolution()?.1;
        let char_max_x_len = max_x_res / FONT.get_width() - 1;
        let char_max_y_len = max_y_res / FONT.get_height() - 1;

        return Ok(Self {
            back_color,
            default_fore_color: fore_color,
            fore_color,
            max_x_res,
            max_y_res,
            char_max_x_len,
            char_max_y_len,
            cursor_x: 0,
            cursor_y: 0,
            target_layer_id: None,
            is_scrollable,
        });
    }

    pub fn init_console(&mut self) -> Result<()> {
        if let Some(layer_id) = &self.target_layer_id {
            let LayerPositionInfo {
                x: _,
                y: _,
                width,
                height,
            } = multi_layer::get_layer_pos_info(layer_id)?;
            self.max_x_res = width;
            self.max_y_res = height;
            self.char_max_x_len = self.max_x_res / FONT.get_width() - 1;
            self.char_max_y_len = self.max_y_res / FONT.get_height() - 1;
        }

        self.cursor_x = 0;
        self.cursor_y = 2;

        self.fill(self.back_color)?;

        for (i, color_code) in GLOBAL_THEME.sample_rect_colors.iter().enumerate() {
            self.draw_rect(i * 20, 0, 20, 20, *color_code)?;
        }

        Ok(())
    }

    pub fn set_target_layer_id(&mut self, layer_id: &LayerId) -> Result<()> {
        self.target_layer_id = Some(layer_id.clone());

        // update
        return self.init_console();
    }

    pub fn set_fore_color(&mut self, fore_color: ColorCode) {
        self.fore_color = fore_color;
    }

    pub fn reset_fore_color(&mut self) {
        self.fore_color = self.default_fore_color;
    }

    pub fn write_char(&mut self, c: char) -> Result<()> {
        match c {
            '\n' => return self.new_line(),
            '\t' => return self.tab(),
            '\x08' | '\x7f' => return self.backspace(),
            _ => (),
        }

        self.draw_font(
            self.cursor_x * FONT.get_width(),
            self.cursor_y * FONT.get_height(),
            c,
            self.fore_color,
            self.back_color,
        )?;

        self.inc_cursor()?;

        Ok(())
    }

    pub fn write_str(&mut self, s: &str) -> Result<()> {
        for c in s.chars() {
            self.write_char(c)?;
        }

        Ok(())
    }

    fn inc_cursor(&mut self) -> Result<()> {
        self.cursor_x += 1;

        if self.cursor_x > self.char_max_x_len {
            self.cursor_x = 0;
            self.cursor_y += 1;
        }

        if self.cursor_y > self.char_max_y_len {
            self.scroll()?;
            self.cursor_x = 0;
            self.cursor_y = if self.is_scrollable {
                self.char_max_y_len
            } else {
                0
            };
        }

        Ok(())
    }

    fn dec_cursor(&mut self) -> Result<()> {
        if self.cursor_x == 0 {
            if self.cursor_y > 0 {
                self.cursor_x = self.char_max_x_len;
                self.cursor_y -= 1;
            }
        } else {
            self.cursor_x -= 1;
        }

        Ok(())
    }

    fn tab(&mut self) -> Result<()> {
        for c in TAB_DISP_STR.chars() {
            self.write_char(c)?;
        }

        Ok(())
    }

    fn new_line(&mut self) -> Result<()> {
        if !self.is_scrollable {
            // fill line
            let font_width = FONT.get_width();
            let font_height = FONT.get_height();
            self.draw_rect(
                self.cursor_x * font_width,
                self.cursor_y * font_height,
                self.max_x_res - self.cursor_x * font_width,
                font_height,
                self.back_color,
            )?;
        }

        self.cursor_x = 0;
        self.cursor_y += 1;

        if self.cursor_y > self.char_max_y_len {
            self.scroll()?;
            self.cursor_y = if self.is_scrollable {
                self.char_max_y_len
            } else {
                0
            };
        }

        Ok(())
    }

    fn scroll(&self) -> Result<()> {
        if !self.is_scrollable {
            return Ok(());
        }

        let font_glyph_size_y = FONT.get_height() - 2;

        for y in font_glyph_size_y..self.max_y_res {
            for x in 0..self.max_x_res {
                self.copy(x, y, x, y - font_glyph_size_y)?;
            }
        }

        Ok(())
    }

    fn fill(&self, color_code: ColorCode) -> Result<()> {
        if let Some(layer_id) = &self.target_layer_id {
            multi_layer::draw_layer(layer_id, |l| l.fill(color_code))?;
        } else {
            frame_buf::fill(color_code)?;
        }

        Ok(())
    }

    fn draw_rect(
        &self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        color_code: ColorCode,
    ) -> Result<()> {
        if let Some(layer_id) = &self.target_layer_id {
            multi_layer::draw_layer(layer_id, |l| l.draw_rect(x, y, width, height, color_code))?;
        } else {
            frame_buf::draw_rect(x, y, width, height, color_code)?;
        }

        Ok(())
    }

    fn draw_font(
        &self,
        x: usize,
        y: usize,
        c: char,
        fore_color_code: ColorCode,
        back_color_code: ColorCode,
    ) -> Result<()> {
        if let Some(layer_id) = &self.target_layer_id {
            multi_layer::draw_layer(layer_id, |l| {
                l.draw_font(x, y, c, fore_color_code, back_color_code)
            })?;
        } else {
            frame_buf::draw_font(x, y, c, fore_color_code, back_color_code)?;
        }

        Ok(())
    }

    fn backspace(&mut self) -> Result<()> {
        self.dec_cursor()?;
        self.draw_rect(
            self.cursor_x * FONT.get_width(),
            self.cursor_y * FONT.get_height(),
            FONT.get_width(),
            FONT.get_height(),
            self.back_color,
        )?;

        Ok(())
    }

    fn copy(&self, x: usize, y: usize, to_x: usize, to_y: usize) -> Result<()> {
        if let Some(layer_id) = &self.target_layer_id {
            multi_layer::draw_layer(layer_id, |l| l.copy(x, y, to_x, to_y))?;
        } else {
            frame_buf::copy(x, y, to_x, to_y)?;
        }

        Ok(())
    }
}

impl fmt::Write for FrameBufferConsole {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let _ = self.write_str(s);
        Ok(())
    }
}

pub fn init(back_color: ColorCode, fore_color: ColorCode, is_scrollable: bool) -> Result<()> {
    let mut fbc = FrameBufferConsole::new(back_color, fore_color, is_scrollable)?;
    fbc.init_console()?;
    *unsafe { FRAME_BUF_CONSOLE.get_force_mut() } = Some(fbc);
    Ok(())
}

pub fn set_target_layer_id(layer_id: &LayerId) -> Result<()> {
    unsafe { FRAME_BUF_CONSOLE.try_lock() }?
        .as_mut()
        .ok_or("FrameBufferConsole is not initialized")?
        .set_target_layer_id(layer_id)
}

pub fn set_fore_color(fore_color: ColorCode) -> Result<()> {
    unsafe { FRAME_BUF_CONSOLE.try_lock() }?
        .as_mut()
        .ok_or("FrameBufferConsole is not initialized")?
        .set_fore_color(fore_color);
    Ok(())
}

pub fn reset_fore_color() -> Result<()> {
    unsafe { FRAME_BUF_CONSOLE.try_lock() }?
        .as_mut()
        .ok_or("FrameBufferConsole is not initialized")?
        .reset_fore_color();
    Ok(())
}

pub fn write_fmt(args: fmt::Arguments) -> Result<()> {
    let _ = unsafe { FRAME_BUF_CONSOLE.try_lock() }?
        .as_mut()
        .ok_or("FrameBufferConsole is not initialized")?
        .write_fmt(args);
    Ok(())
}
