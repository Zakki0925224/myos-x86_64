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
    loop {
        match FRAME_BUF.try_lock() {
            Ok(mut frame_buf) => *frame_buf = Some(FrameBuffer::new(graphic_info)),
            Err(_) => continue,
        }

        match FRAME_BUF_CONSOLE.try_lock() {
            Ok(mut frame_buf_console) => {
                *frame_buf_console = FrameBufferConsole::new(back_color, fore_color)
            }
            Err(_) => continue,
        }

        break;
    }

    info!("graphics: Initialized frame buffer");
}
