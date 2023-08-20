pub mod color;
pub mod font;
pub mod frame_buf;
pub mod frame_buf_console;

use common::graphic_info::GraphicInfo;
use log::info;

use crate::{
    device::serial::SERIAL,
    graphics::{frame_buf::FRAME_BUF, frame_buf_console::FRAME_BUF_CONSOLE},
    util::logger,
};

pub fn init(graphic_info: GraphicInfo) {
    FRAME_BUF.lock().init(graphic_info);
    SERIAL.lock().init();
    FRAME_BUF_CONSOLE.lock().init().unwrap();
    logger::init().unwrap();
    info!("terminal: Initialized kernel terminal");
}
