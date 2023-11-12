use log::error;

pub mod console;
pub mod ps2_keyboard;
pub mod usb;

pub fn init() {
    // initialize ps/2 keyboard
    ps2_keyboard::init();

    // clear console input
    if console::clear_input_buf().is_err() {
        error!("Console is locked");
    }
}
