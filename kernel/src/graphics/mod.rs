pub mod color;
pub mod font;
pub mod frame_buf;
pub mod frame_buf_console;

use common::graphic_info::GraphicInfo;
use log::info;

use crate::graphics::color::RgbColor;

pub fn init(graphic_info: GraphicInfo, back_color: RgbColor, fore_color: RgbColor) {
    if frame_buf::init(graphic_info).is_err() {
        panic!("graphics: Failed to initialize frame buffer");
    }

    if frame_buf_console::init(back_color, fore_color).is_err() {
        panic!("graphics: Failed to initlaize frame buffer console");
    }

    info!("graphics: Initialized frame buffer");
}
