use log::{error, info};

pub mod console;
pub mod ps2_keyboard;
pub mod ps2_mouse;
pub mod usb;

pub fn init() {
    // initialize ps/2 keyboard
    ps2_keyboard::init();
    info!("ps2 kbd: Initialized");

    // initialize ps/2 mouse
    ps2_mouse::init();
    info!("ps2 mouse: Initialized");

    // clear console input
    if console::clear_input_buf().is_err() {
        error!("Console is locked");
    }
}
