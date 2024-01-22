use core::fmt::{self, Write};

use super::{color::COLOR_WHITE, font::PsfFont, frame_buf};
use crate::{
    error::Result,
    graphics::color::*,
    util::mutex::{Mutex, MutexError},
};

const TAB_DISP_CHAR: char = ' ';
const TAB_INDENT_SIZE: usize = 4;

static mut FRAME_BUF_CONSOLE: Mutex<Option<FrameBufferConsole>> = Mutex::new(None);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameBufferConsoleError {
    NotInitialized,
    FontGlyphError,
}

pub struct FrameBufferConsole {
    font: PsfFont,
    back_color: RgbColor,
    default_fore_color: RgbColor,
    fore_color: RgbColor,
    max_x_res: usize,
    max_y_res: usize,
    char_max_x_len: usize,
    char_max_y_len: usize,
    cursor_x: usize,
    cursor_y: usize,
}

impl FrameBufferConsole {
    pub fn new(back_color: RgbColor, fore_color: RgbColor) -> Result<Self> {
        let font = PsfFont::new();
        let max_x_res = frame_buf::get_stride();
        let max_y_res = frame_buf::get_resolution().1;
        let char_max_x_len = max_x_res / font.get_width() - 1;
        let char_max_y_len = max_y_res / font.get_height() - 1;
        let cursor_x = 0;
        let cursor_y = 2;

        frame_buf::clear(&back_color)?;
        frame_buf::draw_rect(0, 0, 20, 20, &COLOR_WHITE)?;
        frame_buf::draw_rect(20, 0, 20, 20, &COLOR_OLIVE)?;
        frame_buf::draw_rect(40, 0, 20, 20, &COLOR_YELLOW)?;
        frame_buf::draw_rect(60, 0, 20, 20, &COLOR_FUCHSIA)?;
        frame_buf::draw_rect(80, 0, 20, 20, &COLOR_SILVER)?;
        frame_buf::draw_rect(100, 0, 20, 20, &COLOR_CYAN)?;
        frame_buf::draw_rect(120, 0, 20, 20, &COLOR_GREEN)?;
        frame_buf::draw_rect(140, 0, 20, 20, &COLOR_RED)?;
        frame_buf::draw_rect(160, 0, 20, 20, &COLOR_GRAY)?;
        frame_buf::draw_rect(180, 0, 20, 20, &COLOR_BLUE)?;
        frame_buf::draw_rect(200, 0, 20, 20, &COLOR_PURPLE)?;
        frame_buf::draw_rect(220, 0, 20, 20, &COLOR_BLACK)?;
        frame_buf::draw_rect(240, 0, 20, 20, &COLOR_NAVY)?;
        frame_buf::draw_rect(260, 0, 20, 20, &COLOR_TEAL)?;
        frame_buf::draw_rect(280, 0, 20, 20, &COLOR_MAROON)?;

        return Ok(Self {
            font,
            back_color,
            default_fore_color: fore_color,
            fore_color,
            max_x_res,
            max_y_res,
            char_max_x_len,
            char_max_y_len,
            cursor_x,
            cursor_y,
        });
    }

    pub fn set_fore_color(&mut self, fore_color: RgbColor) {
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
            self.cursor_x * self.font.get_width(),
            self.cursor_y * self.font.get_height(),
            c,
            &self.fore_color,
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

    fn draw_font<C: Color>(&self, x1: usize, y1: usize, c: char, color: &C) -> Result<()> {
        let glyph = match self
            .font
            .get_glyph(self.font.unicode_char_to_glyph_index(c))
        {
            Some(g) => g,
            None => return Err(FrameBufferConsoleError::FontGlyphError.into()),
        };

        for h in 0..self.font.get_height() {
            for w in 0..self.font.get_width() {
                if !(glyph[h] << w) & 0x80 == 0x80 {
                    continue;
                }

                frame_buf::draw_rect(x1 + w, y1 + h, 1, 1, color)?;
            }
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
        for _ in 0..TAB_INDENT_SIZE {
            self.write_char(TAB_DISP_CHAR)?;
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
        let font_glyph_size_y = self.font.get_height();

        for y in font_glyph_size_y..self.max_y_res {
            for x in 0..self.max_x_res {
                frame_buf::copy_pixel(x, y, x, y - font_glyph_size_y)?;
            }
        }

        frame_buf::draw_rect(
            0,
            self.max_y_res - font_glyph_size_y,
            self.max_x_res - 1,
            font_glyph_size_y - 1,
            &self.back_color,
        )?;

        Ok(())
    }
}

impl fmt::Write for FrameBufferConsole {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s).unwrap();
        Ok(())
    }
}

pub fn init(back_color: RgbColor, fore_color: RgbColor) -> Result<()> {
    if let Ok(mut frame_buf_console) = unsafe { FRAME_BUF_CONSOLE.try_lock() } {
        *frame_buf_console = match FrameBufferConsole::new(back_color, fore_color) {
            Ok(c) => Some(c),
            Err(e) => return Err(e),
        };
        return Ok(());
    }

    Err(MutexError::Locked.into())
}

pub fn set_fore_color(fore_color: RgbColor) -> Result<()> {
    if let Ok(mut frame_buf_console) = unsafe { FRAME_BUF_CONSOLE.try_lock() } {
        if let Some(frame_buf_console) = frame_buf_console.as_mut() {
            frame_buf_console.set_fore_color(fore_color);
            return Ok(());
        } else {
            return Err(FrameBufferConsoleError::NotInitialized.into());
        }
    }

    Err(MutexError::Locked.into())
}

pub fn reset_fore_color() -> Result<()> {
    if let Ok(mut frame_buf_console) = unsafe { FRAME_BUF_CONSOLE.try_lock() } {
        if let Some(frame_buf_console) = frame_buf_console.as_mut() {
            frame_buf_console.reset_fore_color();
            return Ok(());
        } else {
            return Err(FrameBufferConsoleError::NotInitialized.into());
        }
    }

    Err(MutexError::Locked.into())
}

pub fn write_fmt(args: fmt::Arguments) -> Result<()> {
    if let Ok(mut frame_buf_console) = unsafe { FRAME_BUF_CONSOLE.try_lock() } {
        if let Some(frame_buf_console) = frame_buf_console.as_mut() {
            frame_buf_console.write_fmt(args).unwrap();
            return Ok(());
        } else {
            return Err(FrameBufferConsoleError::NotInitialized.into());
        }
    }

    Err(MutexError::Locked.into())
}
