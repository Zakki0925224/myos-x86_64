use self::color::RgbColorCode;
use common::graphic_info::GraphicInfo;
use log::{error, info};

pub mod color;
pub mod draw;
pub mod font;
pub mod frame_buf;
pub mod frame_buf_console;
pub mod multi_layer;

pub fn init(graphic_info: &GraphicInfo, back_color: RgbColorCode, fore_color: RgbColorCode) {
    if let Err(err) = frame_buf::init(graphic_info) {
        panic!("graphics: Failed to initialize frame buffer: {:?}", err);
    }

    if let Err(err) = frame_buf_console::init(back_color, fore_color) {
        panic!(
            "graphics: Failed to initlaize frame buffer console: {:?}",
            err
        );
    }

    info!("graphics: Initialized frame buffer");
}

pub fn enable_shadow_buf() {
    if let Err(err) = frame_buf::enable_shadow_buf() {
        error!("graphics: Failed to enable shadow buffer: {:?}", err);
    }

    info!("graphics: Enabled shadow buffer");
}

pub fn init_layer_man(graphic_info: &GraphicInfo, transparent_color: RgbColorCode) {
    if let Err(err) = multi_layer::init(transparent_color) {
        error!("graphics: Failed to initialize layer manager: {:?}", err);
        return;
    }

    info!("graphics: Initialized layer manager");

    let (res_x, res_y) = graphic_info.resolution;
    let console_layer =
        match multi_layer::create_layer(0, 0, res_x as usize, res_y as usize * 3 / 4) {
            Ok(l) => l,
            Err(err) => {
                error!("graphics: Failed to create the layer: {:?}", err);
                return;
            }
        };
    let console_layer_id = console_layer.id;

    if let Err(err) = multi_layer::push_layer(console_layer) {
        error!(
            "graphics: Failed to configure the layer for the frame buffer console: {:?}",
            err
        );
        return;
    }

    if let Err(err) = frame_buf_console::set_target_layer_id(console_layer_id) {
        error!(
            "graphics: Failed to configure the layer for the frame buffer console: {:?}",
            err
        );
    }
}
