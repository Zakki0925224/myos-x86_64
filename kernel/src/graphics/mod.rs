use self::color::ColorCode;
use common::graphic_info::GraphicInfo;
use log::{error, info};

pub mod color;
pub mod draw;
pub mod font;
pub mod frame_buf;
pub mod frame_buf_console;
pub mod multi_layer;
pub mod simple_window_manager;

pub fn init(graphic_info: &GraphicInfo, back_color: ColorCode, fore_color: ColorCode) {
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

pub fn init_layer_man(graphic_info: &GraphicInfo) {
    let (res_x, res_y) = graphic_info.resolution;
    let console_layer = match multi_layer::create_layer(0, 0, res_x, res_y - 30) {
        Ok(l) => l,
        Err(err) => {
            error!("graphics: Failed to create the layer: {:?}", err);
            return;
        }
    };
    let console_layer_id = console_layer.id.clone();

    if let Err(err) = multi_layer::push_layer(console_layer) {
        error!(
            "graphics: Failed to configure the layer for the frame buffer console: {:?}",
            err
        );
        return;
    }

    if let Err(err) = frame_buf_console::set_target_layer_id(&console_layer_id) {
        error!(
            "graphics: Failed to configure the layer for the frame buffer console: {:?}",
            err
        );
    }
}

pub fn init_simple_wm() {
    if let Err(err) = simple_window_manager::init() {
        error!(
            "graphics: Failed to initialize simple window manager: {:?}",
            err
        );
    }

    if let Err(err) = simple_window_manager::create_taskbar() {
        error!("graphics: Failed to create taskbar: {:?}", err);
    }

    info!("graphics: Initialized simple window manager");
}
