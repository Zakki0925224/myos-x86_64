pub mod color;
pub mod font;
pub mod frame_buffer;
pub mod terminal;

use lazy_static::lazy_static;
use spin::Mutex;

use crate::graphics::{color::RGBColor, frame_buffer::FrameBuffer, terminal::Terminal};

lazy_static! {
    pub static ref FRAME_BUF: Mutex<FrameBuffer> = Mutex::new(FrameBuffer::new());
}

lazy_static! {
    pub static ref TERMINAL: Mutex<Terminal> =
        Mutex::new(Terminal::new(RGBColor::new(3, 26, 0), RGBColor::new(18, 202, 99)));
}
