pub mod color;
pub mod font;
pub mod frame_buf;
pub mod frame_buf_console;

use common::graphic_info::GraphicInfo;
use log::info;

use crate::graphics::{
    color::RgbColor,
    frame_buf_console::{FrameBufferConsole, FRAME_BUF_CONSOLE},
};

pub fn init(graphic_info: GraphicInfo, back_color: RgbColor, fore_color: RgbColor) {
    frame_buf::init(graphic_info);
    match FRAME_BUF_CONSOLE.try_lock() {
        Ok(mut frame_buf_console) => {
            *frame_buf_console = FrameBufferConsole::new(back_color, fore_color)
        }
        Err(_) => panic!(""),
    }

    info!("graphics: Initialized frame buffer");
}
