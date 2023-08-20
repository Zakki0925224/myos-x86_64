pub mod color;
pub mod font;
pub mod frame_buffer;
pub mod terminal;

use common::graphic_info::GraphicInfo;
use lazy_static::lazy_static;
use log::info;
use spin::Mutex;

use crate::{
    device::serial::SERIAL,
    graphics::{color::RGBColor, frame_buffer::FrameBuffer, terminal::Terminal},
    util::logger,
};

lazy_static! {
    pub static ref FRAME_BUF: Mutex<FrameBuffer> = Mutex::new(FrameBuffer::new());
}

lazy_static! {
    pub static ref TERMINAL: Mutex<Terminal> = Mutex::new(Terminal::new(
        RGBColor::new(3, 26, 0),
        RGBColor::new(18, 202, 99)
    ));
}

pub fn init(graphic_info: GraphicInfo) {
    FRAME_BUF.lock().init(graphic_info);
    SERIAL.lock().init();
    TERMINAL.lock().init().unwrap();
    logger::init().unwrap();
    info!("terminal: Initialized kernel terminal");
}
