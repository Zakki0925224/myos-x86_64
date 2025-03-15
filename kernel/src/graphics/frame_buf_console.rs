use super::{
    font::{FONT, TAB_DISP_STR},
    frame_buf,
    multi_layer::{self, LayerId},
};
use crate::{error::Result, theme::GLOBAL_THEME, util::mutex::Mutex, ColorCode};
use core::fmt::{self, Write};

static mut FRAME_BUF_CONSOLE: Mutex<FrameBufferConsole> = Mutex::new(FrameBufferConsole::new());

pub struct FrameBufferConsole {
    back_color: ColorCode,
    default_fore_color: ColorCode,
    fore_color: ColorCode,
    cursor_x: usize,
    cursor_y: usize,
    target_layer_id: Option<LayerId>,
    is_scrollable: bool,
    color_swapped: bool,
}

impl FrameBufferConsole {
    const fn new() -> Self {
        Self {
            back_color: ColorCode::default(),
            default_fore_color: ColorCode::default(),
            fore_color: ColorCode::default(),
            cursor_x: 0,
            cursor_y: 0,
            target_layer_id: None,
            is_scrollable: false,
            color_swapped: false,
        }
    }

    fn screen_wh(&self) -> Result<(usize, usize)> {
        if let Some(layer_id) = &self.target_layer_id {
            let layer_info = multi_layer::get_layer_pos_info(layer_id)?;
            Ok((layer_info.width, layer_info.height))
        } else {
            Ok((frame_buf::get_stride()?, frame_buf::get_resolution()?.1))
        }
    }

    fn init(
        &mut self,
        back_color: ColorCode,
        fore_color: ColorCode,
        is_scrollable: bool,
    ) -> Result<()> {
        self.back_color = back_color;
        self.default_fore_color = fore_color;
        self.fore_color = fore_color;
        self.is_scrollable = is_scrollable;

        self.cursor_x = 0;
        self.cursor_y = 2;

        self.fill(self.back_color)?;

        for (i, color_code) in GLOBAL_THEME.sample_rect_colors.iter().enumerate() {
            self.draw_rect(i * 20, 0, 20, 20, *color_code)?;
        }

        Ok(())
    }

    fn set_target_layer_id(&mut self, layer_id: &LayerId) -> Result<()> {
        self.target_layer_id = Some(layer_id.clone());

        // update
        return self.init(self.back_color, self.fore_color, self.is_scrollable);
    }

    fn set_fore_color(&mut self, fore_color: ColorCode) {
        self.fore_color = fore_color;
    }

    fn reset_fore_color(&mut self) {
        self.fore_color = self.default_fore_color;
    }

    fn write_char(&mut self, c: char) -> Result<()> {
        match c {
            '\n' => return self.new_line(),
            '\t' => return self.tab(),
            '\x08' | '\x7f' => return self.backspace(),
            _ => (),
        }

        let (font_width, font_height) = FONT.get_wh();

        self.draw_font(
            self.cursor_x * font_width,
            self.cursor_y * font_height,
            c,
            self.fore_color,
            self.back_color,
        )?;

        self.inc_cursor()?;

        Ok(())
    }

    fn write_str(&mut self, s: &str) -> Result<()> {
        for c in s.chars() {
            self.write_char(c)?;
        }

        Ok(())
    }

    fn inc_cursor(&mut self) -> Result<()> {
        let (screen_width, screen_height) = self.screen_wh()?;
        let (font_width, font_height) = FONT.get_wh();
        let (char_max_x_len, char_max_y_len) = (
            screen_width / font_width - 1,
            screen_height / font_height - 1,
        );

        self.cursor_x += 1;

        if self.cursor_x > char_max_x_len {
            self.cursor_x = 0;
            self.cursor_y += 1;
        }

        if self.cursor_y > char_max_y_len {
            self.scroll()?;
            self.cursor_x = 0;
            self.cursor_y = if self.is_scrollable {
                char_max_y_len
            } else {
                0
            };
        }

        Ok(())
    }

    fn dec_cursor(&mut self) -> Result<()> {
        let (screen_width, _) = self.screen_wh()?;
        let (font_width, _) = FONT.get_wh();
        let char_max_x_len = screen_width / font_width - 1;

        if self.cursor_x == 0 {
            if self.cursor_y > 0 {
                self.cursor_x = char_max_x_len;
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
        let (screen_width, screen_height) = self.screen_wh()?;
        let (font_width, font_height) = FONT.get_wh();
        let char_max_y_len = screen_height / font_height - 1;

        if !self.is_scrollable {
            // fill line
            self.draw_rect(
                self.cursor_x * font_width,
                self.cursor_y * font_height,
                screen_width - self.cursor_x * font_width,
                font_height,
                self.back_color,
            )?;
        }

        self.cursor_x = 0;
        self.cursor_y += 1;

        if self.cursor_y > char_max_y_len {
            self.scroll()?;
            self.cursor_y = if self.is_scrollable {
                char_max_y_len
            } else {
                0
            };

            // swap color
            if !self.is_scrollable {
                self.color_swapped = !self.color_swapped;
                let tmp = self.default_fore_color;
                if self.color_swapped {
                    self.fore_color = self.back_color;
                    self.default_fore_color = self.back_color;
                    self.back_color = tmp;
                } else {
                    self.fore_color = self.back_color;
                    self.default_fore_color = self.back_color;
                    self.back_color = tmp;
                }
            }
        }

        Ok(())
    }

    fn scroll(&self) -> Result<()> {
        if !self.is_scrollable {
            return Ok(());
        }

        let (_, font_height) = FONT.get_wh();
        let (screen_width, screen_height) = self.screen_wh()?;

        for y in font_height..screen_height {
            for x in 0..screen_width {
                self.copy(x, y, x, y - font_height)?;
            }
        }

        self.draw_rect(
            0,
            screen_height - font_height,
            screen_width,
            font_height,
            self.back_color,
        )?;

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
        fore_color: ColorCode,
        back_color: ColorCode,
    ) -> Result<()> {
        if let Some(layer_id) = &self.target_layer_id {
            multi_layer::draw_layer(layer_id, |l| l.draw_font(x, y, c, fore_color, back_color))?;
        } else {
            frame_buf::draw_font(x, y, c, fore_color, back_color)?;
        }

        Ok(())
    }

    fn backspace(&mut self) -> Result<()> {
        let (font_width, font_height) = FONT.get_wh();

        self.dec_cursor()?;
        self.draw_rect(
            self.cursor_x * font_width,
            self.cursor_y * font_height,
            font_width,
            font_height,
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
    unsafe { FRAME_BUF_CONSOLE.try_lock() }?.init(back_color, fore_color, is_scrollable)
}

pub fn set_target_layer_id(layer_id: &LayerId) -> Result<()> {
    unsafe { FRAME_BUF_CONSOLE.try_lock() }?.set_target_layer_id(layer_id)
}

pub fn set_fore_color(fore_color: ColorCode) -> Result<()> {
    unsafe { FRAME_BUF_CONSOLE.try_lock() }?.set_fore_color(fore_color);
    Ok(())
}

pub fn reset_fore_color() -> Result<()> {
    unsafe { FRAME_BUF_CONSOLE.try_lock() }?.reset_fore_color();
    Ok(())
}

pub fn write_fmt(args: fmt::Arguments) -> Result<()> {
    let _ = unsafe { FRAME_BUF_CONSOLE.try_lock() }?.write_fmt(args);
    Ok(())
}
