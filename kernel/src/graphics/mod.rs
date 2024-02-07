pub mod color;
pub mod draw;
pub mod font;
pub mod frame_buf;
pub mod frame_buf_console;
pub mod multi_layer;

use self::{color::ColorCode, draw::Draw, multi_layer::Layer};
use common::graphic_info::GraphicInfo;
use log::{error, info};

pub fn init(graphic_info: GraphicInfo, back_color: ColorCode, fore_color: ColorCode) {
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
        return;
    }

    info!("graphics: Enabled shadow buffer mode");
}

pub fn init_layer_man(graphic_info: GraphicInfo, transparent_color: ColorCode) {
    if let Err(err) = multi_layer::init(transparent_color) {
        error!("graphics: Failed to initialize layer manager: {:?}", err);
        return;
    }

    // layer for frame buffer console
    let mut layer = match Layer::new(200, 50, 600, 300, graphic_info.format) {
        Ok(l) => l,
        Err(err) => {
            error!("graphics: Failed to create the layer: {:?}", err);
            return;
        }
    };

    if let Err(err) = layer.fill(transparent_color) {
        error!("graphics: Fialed to initialize the layer: {:?}", err);
        return;
    }

    let layer_id = layer.id;

    if let Err(err) = multi_layer::push_layer(layer) {
        error!(
            "graphics: Failed to configure the layer for the frame buffer console: {:?}",
            err
        );
        return;
    }

    if let Err(err) = frame_buf_console::set_target_layer_id(layer_id) {
        error!(
            "graphics: Failed to configure the layer for the frame buffer console: {:?}",
            err
        );
    }

    info!(
        "graphics: Configured frame buffer console to use layer #{}",
        layer_id
    );
}
