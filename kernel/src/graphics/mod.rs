use self::color::ColorCode;
use crate::error::Result;
use common::graphic_info::GraphicInfo;
use log::info;

pub mod color;
pub mod draw;
pub mod font;
pub mod frame_buf;
pub mod frame_buf_console;
pub mod multi_layer;
pub mod simple_window_manager;

pub fn init(
    graphic_info: &GraphicInfo,
    back_color: ColorCode,
    fore_color: ColorCode,
) -> Result<()> {
    frame_buf::init(graphic_info)?;
    frame_buf_console::init(back_color, fore_color)?;

    info!("graphics: Initialized frame buffer");
    Ok(())
}

pub fn enable_shadow_buf() -> Result<()> {
    frame_buf::enable_shadow_buf()?;

    info!("graphics: Enabled shadow buffer");
    Ok(())
}

pub fn init_layer_man(graphic_info: &GraphicInfo) -> Result<()> {
    let (res_x, res_y) = graphic_info.resolution;
    let console_layer = multi_layer::create_layer(0, 0, res_x, res_y - 30)?;
    let console_layer_id = console_layer.id.clone();

    multi_layer::push_layer(console_layer)?;
    frame_buf_console::set_target_layer_id(&console_layer_id)?;

    info!("graphics: Initialized layer manager");
    Ok(())
}

pub fn init_simple_wm() -> Result<()> {
    simple_window_manager::init()?;
    simple_window_manager::create_taskbar()?;

    info!("graphics: Initialized simple window manager");
    Ok(())
}
