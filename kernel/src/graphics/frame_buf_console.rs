use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

use crate::graphics::color::*;

use super::{
    color::COLOR_WHITE,
    font::PsfFont,
    frame_buf::{FrameBufferError, FRAME_BUF},
};

const TAB_DISP_CHAR: char = ' ';
const TAB_INDENT_SIZE: usize = 4;

lazy_static! {
    pub static ref FRAME_BUF_CONSOLE: Mutex<FrameBufferConsole> = Mutex::new(
        FrameBufferConsole::new(RgbColor::new(3, 26, 0), RgbColor::new(18, 202, 99))
    );
}

#[derive(Debug)]
pub enum FrameBufferConsoleError {
    NotInitialized,
    FontGlyphError,
    FrameBufferError(FrameBufferError),
}

pub struct FrameBufferConsole {
    is_init: bool,
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
    pub fn new(back_color: RgbColor, fore_color: RgbColor) -> Self {
        return Self {
            is_init: false,
            font: PsfFont::new(),
            back_color,
            default_fore_color: fore_color,
            fore_color,
            max_x_res: 0,
            max_y_res: 0,
            char_max_x_len: 0,
            char_max_y_len: 0,
            cursor_x: 0,
            cursor_y: 0,
        };
    }

    pub fn init(&mut self) -> Result<(), FrameBufferConsoleError> {
        if !FRAME_BUF.lock().is_init() {
            return Err(FrameBufferConsoleError::FrameBufferError(
                FrameBufferError::NotInitialized,
            ));
        }

        self.max_x_res = FRAME_BUF.lock().get_stride();
        self.max_y_res = FRAME_BUF.lock().get_resolution().1;
        self.char_max_x_len = self.max_x_res / self.font.get_width() - 1;
        self.char_max_y_len = self.max_y_res / self.font.get_height() - 1;
        self.cursor_x = 0;
        self.cursor_y = 2;

        // TODO
        FRAME_BUF.lock().clear(&self.back_color).unwrap();
        FRAME_BUF
            .lock()
            .draw_rect(0, 0, 20, 20, &COLOR_WHITE)
            .unwrap();
        FRAME_BUF
            .lock()
            .draw_rect(20, 0, 20, 20, &COLOR_OLIVE)
            .unwrap();
        FRAME_BUF
            .lock()
            .draw_rect(40, 0, 20, 20, &COLOR_YELLOW)
            .unwrap();
        FRAME_BUF
            .lock()
            .draw_rect(60, 0, 20, 20, &COLOR_FUCHSIA)
            .unwrap();
        FRAME_BUF
            .lock()
            .draw_rect(80, 0, 20, 20, &COLOR_SILVER)
            .unwrap();
        FRAME_BUF
            .lock()
            .draw_rect(100, 0, 20, 20, &COLOR_CYAN)
            .unwrap();
        FRAME_BUF
            .lock()
            .draw_rect(120, 0, 20, 20, &COLOR_GREEN)
            .unwrap();
        FRAME_BUF
            .lock()
            .draw_rect(140, 0, 20, 20, &COLOR_RED)
            .unwrap();
        FRAME_BUF
            .lock()
            .draw_rect(160, 0, 20, 20, &COLOR_GRAY)
            .unwrap();
        FRAME_BUF
            .lock()
            .draw_rect(180, 0, 20, 20, &COLOR_BLUE)
            .unwrap();
        FRAME_BUF
            .lock()
            .draw_rect(200, 0, 20, 20, &COLOR_PURPLE)
            .unwrap();
        FRAME_BUF
            .lock()
            .draw_rect(220, 0, 20, 20, &COLOR_BLACK)
            .unwrap();
        FRAME_BUF
            .lock()
            .draw_rect(240, 0, 20, 20, &COLOR_NAVY)
            .unwrap();
        FRAME_BUF
            .lock()
            .draw_rect(260, 0, 20, 20, &COLOR_TEAL)
            .unwrap();
        FRAME_BUF
            .lock()
            .draw_rect(280, 0, 20, 20, &COLOR_MAROON)
            .unwrap();

        self.is_init = true;

        return Ok(());
    }

    pub fn set_fore_color(&mut self, fore_color: RgbColor) {
        self.fore_color = fore_color;
    }

    pub fn reset_fore_color(&mut self) {
        self.fore_color = self.default_fore_color;
    }

    pub fn write_char(&mut self, c: char) -> Result<(), FrameBufferConsoleError> {
        if !self.is_init {
            return Err(FrameBufferConsoleError::NotInitialized);
        }

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

        return self.inc_cursor();
    }

    pub fn write_string(&mut self, string: &str) -> Result<(), FrameBufferConsoleError> {
        for c in string.chars() {
            if let Err(err) = self.write_char(c) {
                return Err(err);
            }
        }

        return Ok(());
    }

    fn draw_font(
        &self,
        x1: usize,
        y1: usize,
        c: char,
        color: &impl Color,
    ) -> Result<(), FrameBufferConsoleError> {
        let glyph = match self
            .font
            .get_glyph(self.font.unicode_char_to_glyph_index(c))
        {
            Some(g) => g,
            None => return Err(FrameBufferConsoleError::FontGlyphError),
        };

        for h in 0..self.font.get_height() {
            for w in 0..self.font.get_width() {
                if !(glyph[h] << w) & 0x80 == 0x80 {
                    continue;
                }

                if let Err(err) = FRAME_BUF.lock().draw_rect(x1 + w, y1 + h, 1, 1, color) {
                    return Err(FrameBufferConsoleError::FrameBufferError(err));
                }
            }
        }

        return Ok(());
    }

    fn inc_cursor(&mut self) -> Result<(), FrameBufferConsoleError> {
        self.cursor_x += 1;

        if self.cursor_x > self.char_max_x_len {
            self.cursor_x = 0;
            self.cursor_y += 1;
        }

        if self.cursor_y > self.char_max_y_len {
            if let Err(err) = self.scroll() {
                return Err(err);
            }
            self.cursor_x = 0;
            self.cursor_y = self.char_max_y_len;
        }

        return Ok(());
    }

    fn tab(&mut self) -> Result<(), FrameBufferConsoleError> {
        for _ in 0..TAB_INDENT_SIZE {
            if let Err(err) = self.write_char(TAB_DISP_CHAR) {
                return Err(err);
            }

            if let Err(err) = self.inc_cursor() {
                return Err(err);
            }
        }

        return Ok(());
    }

    fn new_line(&mut self) -> Result<(), FrameBufferConsoleError> {
        self.cursor_x = 0;
        self.cursor_y += 1;

        if self.cursor_y > self.char_max_y_len {
            if let Err(err) = self.scroll() {
                return Err(err);
            }
            self.cursor_y = self.char_max_y_len;
        }

        return Ok(());
    }

    fn scroll(&self) -> Result<(), FrameBufferConsoleError> {
        let font_glyph_size_y = self.font.get_height();

        for y in font_glyph_size_y..self.max_y_res {
            for x in 0..self.max_x_res {
                if let Err(err) = FRAME_BUF.lock().copy_pixel(x, y, x, y - font_glyph_size_y) {
                    return Err(FrameBufferConsoleError::FrameBufferError(err));
                }
            }
        }

        if let Err(err) = FRAME_BUF.lock().draw_rect(
            0,
            self.max_y_res - font_glyph_size_y,
            self.max_x_res - 1,
            font_glyph_size_y - 1,
            &self.back_color,
        ) {
            return Err(FrameBufferConsoleError::FrameBufferError(err));
        }

        return Ok(());
    }
}

impl fmt::Write for FrameBufferConsole {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if !self.is_init {
            panic!("console: {:?}", FrameBufferConsoleError::NotInitialized);
        }

        self.write_string(s).unwrap();
        return Ok(());
    }
}
