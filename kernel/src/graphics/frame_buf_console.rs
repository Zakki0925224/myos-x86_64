use super::{
    font::{FONT, TAB_DISP_STR},
    frame_buf,
    multi_layer::{self, LayerPositionInfo},
};
use crate::{
    error::Result,
    graphics::color::*,
    util::mutex::{Mutex, MutexError},
};
use core::fmt::{self, Write};

static mut FRAME_BUF_CONSOLE: Mutex<Option<FrameBufferConsole>> = Mutex::new(None);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameBufferConsoleError {
    NotInitialized,
}

pub struct FrameBufferConsole {
    back_color: RgbColorCode,
    default_fore_color: RgbColorCode,
    fore_color: RgbColorCode,
    max_x_res: usize,
    max_y_res: usize,
    char_max_x_len: usize,
    char_max_y_len: usize,
    cursor_x: usize,
    cursor_y: usize,
    target_layer_id: Option<usize>,
}

impl FrameBufferConsole {
    pub fn new(back_color: RgbColorCode, fore_color: RgbColorCode) -> Result<Self> {
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
        });
    }

    pub fn init_console(&mut self) -> Result<()> {
        if let Some(layer_id) = self.target_layer_id {
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

        self.draw_rect(0, 0, 20, 20, COLOR_WHITE)?;
        self.draw_rect(20, 0, 20, 20, COLOR_OLIVE)?;
        self.draw_rect(40, 0, 20, 20, COLOR_YELLOW)?;
        self.draw_rect(60, 0, 20, 20, COLOR_FUCHSIA)?;
        self.draw_rect(80, 0, 20, 20, COLOR_SILVER)?;
        self.draw_rect(100, 0, 20, 20, COLOR_CYAN)?;
        self.draw_rect(120, 0, 20, 20, COLOR_GREEN)?;
        self.draw_rect(140, 0, 20, 20, COLOR_RED)?;
        self.draw_rect(160, 0, 20, 20, COLOR_GRAY)?;
        self.draw_rect(180, 0, 20, 20, COLOR_BLUE)?;
        self.draw_rect(200, 0, 20, 20, COLOR_PURPLE)?;
        self.draw_rect(220, 0, 20, 20, COLOR_BLACK)?;
        self.draw_rect(240, 0, 20, 20, COLOR_NAVY)?;
        self.draw_rect(260, 0, 20, 20, COLOR_TEAL)?;
        self.draw_rect(280, 0, 20, 20, COLOR_MAROON)?;

        Ok(())
    }

    pub fn set_target_layer_id(&mut self, layer_id: usize) -> Result<()> {
        self.target_layer_id = Some(layer_id);

        // update
        return self.init_console();
    }

    pub fn set_fore_color(&mut self, fore_color: RgbColorCode) {
        self.fore_color = fore_color;
    }

    pub fn reset_fore_color(&mut self) {
        self.fore_color = self.default_fore_color;
    }

    pub fn write_char(&mut self, c: char) -> Result<()> {
        match c {
            '\n' => return self.new_line(),
            '\t' => return self.tab(),
            _ => (),
        }

        self.draw_font(
            self.cursor_x * FONT.get_width(),
            self.cursor_y * FONT.get_height(),
            c,
            self.fore_color,
        )?;

        self.inc_cursor()?;

        Ok(())
    }

    pub fn write_string(&mut self, string: &str) -> Result<()> {
        for c in string.chars() {
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
            self.cursor_y = self.char_max_y_len;
        }

        Ok(())
    }

    fn tab(&mut self) -> Result<()> {
        for c in TAB_DISP_STR.chars() {
            self.write_char(c)?;
            self.inc_cursor()?;
        }

        Ok(())
    }

    fn new_line(&mut self) -> Result<()> {
        self.cursor_x = 0;
        self.cursor_y += 1;

        if self.cursor_y > self.char_max_y_len {
            self.scroll()?;
            self.cursor_y = self.char_max_y_len;
        }

        Ok(())
    }

    fn scroll(&self) -> Result<()> {
        let font_glyph_size_y = FONT.get_height();

        for y in font_glyph_size_y..self.max_y_res {
            for x in 0..self.max_x_res {
                self.copy(x, y, x, y - font_glyph_size_y)?;
            }
        }

        self.draw_rect(
            0,
            self.max_y_res - font_glyph_size_y,
            self.max_x_res,
            font_glyph_size_y,
            self.back_color,
        )?;

        Ok(())
    }

    fn fill(&self, color_code: RgbColorCode) -> Result<()> {
        if let Some(layer_id) = self.target_layer_id {
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
        color_code: RgbColorCode,
    ) -> Result<()> {
        if let Some(layer_id) = self.target_layer_id {
            multi_layer::draw_layer(layer_id, |l| l.draw_rect(x, y, width, height, color_code))?;
        } else {
            frame_buf::draw_rect(x, y, width, height, color_code)?;
        }

        Ok(())
    }

    fn draw_font(&self, x: usize, y: usize, c: char, color_code: RgbColorCode) -> Result<()> {
        if let Some(layer_id) = self.target_layer_id {
            multi_layer::draw_layer(layer_id, |l| l.draw_font(x, y, c, color_code))?;
        } else {
            frame_buf::draw_font(x, y, c, color_code)?;
        }

        Ok(())
    }

    fn copy(&self, x: usize, y: usize, to_x: usize, to_y: usize) -> Result<()> {
        if let Some(layer_id) = self.target_layer_id {
            multi_layer::draw_layer(layer_id, |l| l.copy(x, y, to_x, to_y))?;
        } else {
            frame_buf::copy(x, y, to_x, to_y)?;
        }

        Ok(())
    }
}

impl fmt::Write for FrameBufferConsole {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let _ = self.write_string(s);
        Ok(())
    }
}

pub fn init(back_color: RgbColorCode, fore_color: RgbColorCode) -> Result<()> {
    if let Ok(mut frame_buf_console) = unsafe { FRAME_BUF_CONSOLE.try_lock() } {
        *frame_buf_console = match FrameBufferConsole::new(back_color, fore_color) {
            Ok(c) => Some(c),
            Err(e) => return Err(e),
        };

        frame_buf_console.as_mut().unwrap().init_console()?;
        return Ok(());
    }

    Err(MutexError::Locked.into())
}

pub fn set_target_layer_id(layer_id: usize) -> Result<()> {
    if let Ok(mut frame_buf_console) = unsafe { FRAME_BUF_CONSOLE.try_lock() } {
        if let Some(frame_buf_console) = frame_buf_console.as_mut() {
            return frame_buf_console.set_target_layer_id(layer_id);
        }

        return Err(FrameBufferConsoleError::NotInitialized.into());
    }

    Err(MutexError::Locked.into())
}

pub fn set_fore_color(fore_color: RgbColorCode) -> Result<()> {
    if let Ok(mut frame_buf_console) = unsafe { FRAME_BUF_CONSOLE.try_lock() } {
        if let Some(frame_buf_console) = frame_buf_console.as_mut() {
            frame_buf_console.set_fore_color(fore_color);
            return Ok(());
        }

        return Err(FrameBufferConsoleError::NotInitialized.into());
    }

    Err(MutexError::Locked.into())
}

pub fn reset_fore_color() -> Result<()> {
    if let Ok(mut frame_buf_console) = unsafe { FRAME_BUF_CONSOLE.try_lock() } {
        if let Some(frame_buf_console) = frame_buf_console.as_mut() {
            frame_buf_console.reset_fore_color();
            return Ok(());
        }

        return Err(FrameBufferConsoleError::NotInitialized.into());
    }

    Err(MutexError::Locked.into())
}

pub fn write_fmt(args: fmt::Arguments) -> Result<()> {
    if let Ok(mut frame_buf_console) = unsafe { FRAME_BUF_CONSOLE.try_lock() } {
        if let Some(frame_buf_console) = frame_buf_console.as_mut() {
            let _ = frame_buf_console.write_fmt(args);
            return Ok(());
        }

        return Err(FrameBufferConsoleError::NotInitialized.into());
    }

    Err(MutexError::Locked.into())
}
