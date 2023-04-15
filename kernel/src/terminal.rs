use crate::{graphics::color::*, serial::SERIAL};

use super::graphics::FRAME_BUF;
use core::fmt::{self, Write};
use lazy_static::lazy_static;
use spin::Mutex;

const TAB_DISP_CHAR: char = ' ';
const TAB_INDENT_SIZE: usize = 4;

lazy_static! {
    pub static ref TERMINAL: Mutex<Terminal> =
        Mutex::new(Terminal::new(RGBColor::new(3, 26, 0), RGBColor::new(18, 202, 99)));
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

    pub fn init(&mut self)
    {
        if !FRAME_BUF.lock().is_init()
        {
            panic!("Graphics is not initialized");
        }

        let (glyph_size_width, _) = FRAME_BUF.lock().get_font_glyph_size();
        self.font_glyph_size = (glyph_size_width, 16);

        self.max_x_res = FRAME_BUF.lock().get_stride();
        self.max_y_res = FRAME_BUF.lock().get_resolution().1;
        self.char_max_x_len = self.max_x_res / self.font_glyph_size.0 - 1;
        self.char_max_y_len = self.max_y_res / self.font_glyph_size.1 - 1;
        self.cursor_x = 0;
        self.cursor_y = 2;

        FRAME_BUF.lock().clear(&self.back_color);
        FRAME_BUF.lock().draw_rect(0, 0, 20, 20, &COLOR_WHITE);
        FRAME_BUF.lock().draw_rect(20, 0, 20, 20, &COLOR_OLIVE);
        FRAME_BUF.lock().draw_rect(40, 0, 20, 20, &COLOR_YELLOW);
        FRAME_BUF.lock().draw_rect(60, 0, 20, 20, &COLOR_FUCHSIA);
        FRAME_BUF.lock().draw_rect(80, 0, 20, 20, &COLOR_SILVER);
        FRAME_BUF.lock().draw_rect(100, 0, 20, 20, &COLOR_CYAN);
        FRAME_BUF.lock().draw_rect(120, 0, 20, 20, &COLOR_GREEN);
        FRAME_BUF.lock().draw_rect(140, 0, 20, 20, &COLOR_RED);
        FRAME_BUF.lock().draw_rect(160, 0, 20, 20, &COLOR_GRAY);
        FRAME_BUF.lock().draw_rect(180, 0, 20, 20, &COLOR_BLUE);
        FRAME_BUF.lock().draw_rect(200, 0, 20, 20, &COLOR_PURPLE);
        FRAME_BUF.lock().draw_rect(220, 0, 20, 20, &COLOR_BLACK);
        FRAME_BUF.lock().draw_rect(240, 0, 20, 20, &COLOR_NAVY);
        FRAME_BUF.lock().draw_rect(260, 0, 20, 20, &COLOR_TEAL);
        FRAME_BUF.lock().draw_rect(280, 0, 20, 20, &COLOR_MAROON);

        self.is_init = true;
    }

    pub fn set_fore_color(&mut self, fore_color: RGBColor) { self.fore_color = fore_color; }

    pub fn reset_fore_color(&mut self) { self.fore_color = self.default_fore_color; }

    pub fn write_char(&mut self, c: char)
    {
        if !self.is_init
        {
            panic!("Terminal is not initialized");
        }

        match c
        {
            '\n' =>
            {
                self.new_line();
                return;
            }
            '\t' =>
            {
                self.tab();
                return;
            }
            _ => (),
        }

        FRAME_BUF.lock().draw_font(
            self.cursor_x * self.font_glyph_size.0,
            self.cursor_y * self.font_glyph_size.1,
            c,
            &self.fore_color,
        );

        // TODO: send Terminal color code
        SERIAL.lock().send_data(c as u8);

        self.inc_cursor();
    }

    pub fn write_string(&mut self, string: &str)
    {
        for c in string.chars()
        {
            self.write_char(c);
        }
    }

    fn inc_cursor(&mut self)
    {
        self.cursor_x += 1;

        if self.cursor_x > self.char_max_x_len
        {
            self.cursor_x = 0;
            self.cursor_y += 1;
        }

        if self.cursor_y > self.char_max_y_len
        {
            self.scroll();
            self.cursor_x = 0;
            self.cursor_y = self.char_max_y_len;
        }
    }

    fn tab(&mut self)
    {
        for _ in 0..TAB_INDENT_SIZE
        {
            self.write_char(TAB_DISP_CHAR);
            self.inc_cursor();
        }

        SERIAL.lock().send_data(b'\t');
    }

    fn new_line(&mut self)
    {
        self.cursor_x = 0;
        self.cursor_y += 1;

        if self.cursor_y > self.char_max_y_len
        {
            self.scroll();
            self.cursor_y = self.char_max_y_len;
        }

        SERIAL.lock().send_data(b'\n');
    }

    // scroll is too slow -> use KVM
    fn scroll(&self)
    {
        let font_glyph_size_y = self.font_glyph_size.1;

        for y in font_glyph_size_y..self.max_y_res
        {
            for x in 0..self.max_x_res
            {
                FRAME_BUF.lock().copy_pixel(x, y, x, y - font_glyph_size_y);
            }
        }

        for y in self.max_y_res - font_glyph_size_y..self.max_y_res
        {
            for x in 0..self.max_x_res
            {
                FRAME_BUF.lock().set_color(x, y, &self.back_color);
            }
        }
    }
}

impl fmt::Write for Terminal
{
    fn write_str(&mut self, s: &str) -> fmt::Result
    {
        if !self.is_init
        {
            panic!("Terminal is not initialized");
        }

        self.write_string(s);
        return Ok(());
    }
}

// print!, println! macro
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) { TERMINAL.lock().write_fmt(args).unwrap(); }

#[macro_export]
macro_rules! print
{
    ($($arg:tt)*) => ($crate::terminal::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println
{
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
