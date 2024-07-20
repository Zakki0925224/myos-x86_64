use crate::error::Result;
use log::{error, info};

pub mod console;
pub mod ps2_keyboard;
pub mod ps2_mouse;
pub mod serial;
pub mod usb;
pub mod virtio;

trait DeviceDriverFunction {
    // check and find device
    fn probe(&mut self) -> Result<()>;
    // initialize device
    fn attach(&mut self) -> Result<()>;
}

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

    if let Err(err) = virtio::net::probe_and_attach() {
        error!("vtnet: Failed to probe and attach device: {:?}", err);
    }
}
