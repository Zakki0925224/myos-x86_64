pub mod color;
pub mod font;
pub mod frame_buf;
pub mod frame_buf_console;

use common::graphic_info::GraphicInfo;
use log::info;

use crate::graphics::{
    color::RgbColor,
    frame_buf::{FrameBuffer, FRAME_BUF},
    frame_buf_console::{FrameBufferConsole, FRAME_BUF_CONSOLE},
};

pub fn init(graphic_info: GraphicInfo, back_color: RgbColor, fore_color: RgbColor) {
    if let Some(mut frame_buf) = FRAME_BUF.try_lock() {
        *frame_buf = Some(FrameBuffer::new(graphic_info));
    }

    if let Some(mut frame_buf_console) = FRAME_BUF_CONSOLE.try_lock() {
        *frame_buf_console = FrameBufferConsole::new(back_color, fore_color);
    }

    info!("graphics: Initialized frame buffer");
}
