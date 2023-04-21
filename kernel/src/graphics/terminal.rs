use crate::{graphics::{color::*, frame_buffer::FrameBufferError}, serial::{SerialPortError, SERIAL}};
use core::fmt::{self, Write};

use super::{FRAME_BUF, TERMINAL};

const TAB_DISP_CHAR: char = ' ';
const TAB_INDENT_SIZE: usize = 4;
#[derive(Debug)]
pub enum TerminalError
{
    NotInitialized,
    FrameBufferError(FrameBufferError),
    SerialPortError(SerialPortError),
}

pub struct Terminal
{
    is_init: bool,
    back_color: RGBColor,
    default_fore_color: RGBColor,
    fore_color: RGBColor,
    font_glyph_size: (usize, usize),
    max_x_res: usize,
    max_y_res: usize,
    char_max_x_len: usize,
    char_max_y_len: usize,
    cursor_x: usize,
    cursor_y: usize,
}

impl Terminal
{
    pub fn new(back_color: RGBColor, fore_color: RGBColor) -> Self
    {
        return Self {
            is_init: false,
            back_color,
            default_fore_color: fore_color,
            fore_color,
            font_glyph_size: (0, 0),
            max_x_res: 0,
            max_y_res: 0,
            char_max_x_len: 0,
            char_max_y_len: 0,
            cursor_x: 0,
            cursor_y: 0,
        };
    }

    pub fn init(&mut self) -> Result<(), TerminalError>
    {
        if !FRAME_BUF.lock().is_init()
        {
            return Err(TerminalError::FrameBufferError(FrameBufferError::NotInitialized));
        }

        let (glyph_size_width, _) = FRAME_BUF.lock().get_font_glyph_size();
        self.font_glyph_size = (glyph_size_width, 16);

        self.max_x_res = FRAME_BUF.lock().get_stride();
        self.max_y_res = FRAME_BUF.lock().get_resolution().1;
        self.char_max_x_len = self.max_x_res / self.font_glyph_size.0 - 1;
        self.char_max_y_len = self.max_y_res / self.font_glyph_size.1 - 1;
        self.cursor_x = 0;
        self.cursor_y = 2;

        // TODO
        FRAME_BUF.lock().clear(&self.back_color).unwrap();
        FRAME_BUF.lock().draw_rect(0, 0, 20, 20, &COLOR_WHITE).unwrap();
        FRAME_BUF.lock().draw_rect(20, 0, 20, 20, &COLOR_OLIVE).unwrap();
        FRAME_BUF.lock().draw_rect(40, 0, 20, 20, &COLOR_YELLOW).unwrap();
        FRAME_BUF.lock().draw_rect(60, 0, 20, 20, &COLOR_FUCHSIA).unwrap();
        FRAME_BUF.lock().draw_rect(80, 0, 20, 20, &COLOR_SILVER).unwrap();
        FRAME_BUF.lock().draw_rect(100, 0, 20, 20, &COLOR_CYAN).unwrap();
        FRAME_BUF.lock().draw_rect(120, 0, 20, 20, &COLOR_GREEN).unwrap();
        FRAME_BUF.lock().draw_rect(140, 0, 20, 20, &COLOR_RED).unwrap();
        FRAME_BUF.lock().draw_rect(160, 0, 20, 20, &COLOR_GRAY).unwrap();
        FRAME_BUF.lock().draw_rect(180, 0, 20, 20, &COLOR_BLUE).unwrap();
        FRAME_BUF.lock().draw_rect(200, 0, 20, 20, &COLOR_PURPLE).unwrap();
        FRAME_BUF.lock().draw_rect(220, 0, 20, 20, &COLOR_BLACK).unwrap();
        FRAME_BUF.lock().draw_rect(240, 0, 20, 20, &COLOR_NAVY).unwrap();
        FRAME_BUF.lock().draw_rect(260, 0, 20, 20, &COLOR_TEAL).unwrap();
        FRAME_BUF.lock().draw_rect(280, 0, 20, 20, &COLOR_MAROON).unwrap();

        self.is_init = true;

        return Ok(());
    }

    pub fn set_fore_color(&mut self, fore_color: RGBColor) { self.fore_color = fore_color; }

    pub fn reset_fore_color(&mut self) { self.fore_color = self.default_fore_color; }

    pub fn write_char(&mut self, c: char) -> Result<(), TerminalError>
    {
        if !self.is_init
        {
            return Err(TerminalError::NotInitialized);
        }

        match c
        {
            '\n' => return self.new_line(),
            '\t' => return self.tab(),
            _ => (),
        }

        if let Err(err) = FRAME_BUF.lock().draw_font(
            self.cursor_x * self.font_glyph_size.0,
            self.cursor_y * self.font_glyph_size.1,
            c,
            &self.fore_color,
        )
        {
            return Err(TerminalError::FrameBufferError(err));
        }

        // TODO: send Terminal color code
        if let Err(err) = SERIAL.lock().send_data(c as u8)
        {
            return Err(TerminalError::SerialPortError(err));
        }

        return self.inc_cursor();
    }

    pub fn write_string(&mut self, string: &str) -> Result<(), TerminalError>
    {
        for c in string.chars()
        {
            if let Err(err) = self.write_char(c)
            {
                return Err(err);
            }
        }

        return Ok(());
    }

    fn inc_cursor(&mut self) -> Result<(), TerminalError>
    {
        self.cursor_x += 1;

        if self.cursor_x > self.char_max_x_len
        {
            self.cursor_x = 0;
            self.cursor_y += 1;
        }

        if self.cursor_y > self.char_max_y_len
        {
            if let Err(err) = self.scroll()
            {
                return Err(err);
            }
            self.cursor_x = 0;
            self.cursor_y = self.char_max_y_len;
        }

        return Ok(());
    }

    fn tab(&mut self) -> Result<(), TerminalError>
    {
        for _ in 0..TAB_INDENT_SIZE
        {
            if let Err(err) = self.write_char(TAB_DISP_CHAR)
            {
                return Err(err);
            }

            if let Err(err) = self.inc_cursor()
            {
                return Err(err);
            }
        }

        if let Err(err) = SERIAL.lock().send_data(b'\t')
        {
            return Err(TerminalError::SerialPortError(err));
        }

        return Ok(());
    }

    fn new_line(&mut self) -> Result<(), TerminalError>
    {
        self.cursor_x = 0;
        self.cursor_y += 1;

        if self.cursor_y > self.char_max_y_len
        {
            if let Err(err) = self.scroll()
            {
                return Err(err);
            }
            self.cursor_y = self.char_max_y_len;
        }

        if let Err(err) = SERIAL.lock().send_data(b'\n')
        {
            return Err(TerminalError::SerialPortError(err));
        }

        return Ok(());
    }

    fn scroll(&self) -> Result<(), TerminalError>
    {
        let font_glyph_size_y = self.font_glyph_size.1;

        for y in font_glyph_size_y..self.max_y_res
        {
            for x in 0..self.max_x_res
            {
                if let Err(err) = FRAME_BUF.lock().copy_pixel(x, y, x, y - font_glyph_size_y)
                {
                    return Err(TerminalError::FrameBufferError(err));
                }
            }
        }

        if let Err(err) = FRAME_BUF.lock().draw_rect(
            0,
            self.max_y_res - font_glyph_size_y,
            self.max_x_res - 1,
            font_glyph_size_y - 1,
            &self.back_color,
        )
        {
            return Err(TerminalError::FrameBufferError(err));
        }

        return Ok(());
    }
}

impl fmt::Write for Terminal
{
    fn write_str(&mut self, s: &str) -> fmt::Result
    {
        if !self.is_init
        {
            panic!("terminal: {:?}", TerminalError::NotInitialized);
        }

        self.write_string(s).unwrap();
        return Ok(());
    }
}

// print!, println! macro
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) { TERMINAL.lock().write_fmt(args).unwrap(); }

#[macro_export]
macro_rules! print
{
    ($($arg:tt)*) => ($crate::graphics::terminal::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println
{
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
