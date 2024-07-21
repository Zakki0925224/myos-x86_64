use crate::error::Result;
use log::{error, info};

pub mod console;
pub mod ps2_keyboard;
pub mod ps2_mouse;
pub mod serial;
pub mod usb;
pub mod virtio;

#[derive(Debug, Clone)]
pub struct DeviceDriverInfo {
    name: &'static str,
    attached: bool,
}

impl DeviceDriverInfo {
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            attached: false,
        }
    }
}

trait DeviceDriverFunction {
    fn get_device_driver_info(&self) -> Result<DeviceDriverInfo>;
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
    info!(
        "vtnet: Attached: {:?}",
        virtio::net::get_device_driver_info()
    );
}
