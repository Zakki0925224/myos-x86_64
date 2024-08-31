use crate::error::Result;
use log::error;

pub mod console;
pub mod ps2_keyboard;
pub mod ps2_mouse;
pub mod uart;
pub mod usb;
pub mod virtio;

#[derive(Debug, Clone)]
pub struct DeviceDriverInfo {
    pub name: &'static str,
    pub attached: bool,
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
    type PollNormalOutput;
    type PollInterruptOutput;

    fn get_device_driver_info(&self) -> Result<DeviceDriverInfo>;
    // check and find device
    fn probe(&mut self) -> Result<()>;
    // initialize device
    fn attach(&mut self) -> Result<()>;
    // normal polling
    fn poll_normal(&mut self) -> Result<Self::PollNormalOutput>;
    // interrupt polling
    fn poll_int(&mut self) -> Result<Self::PollInterruptOutput>;
}

pub fn init() {
    if let Err(err) = ps2_keyboard::probe_and_attach() {
        let name = ps2_keyboard::get_device_driver_info().unwrap().name;
        error!("{}: Failed to probe or attach device: {:?}", name, err);
    }

    if let Err(err) = ps2_mouse::probe_and_attach() {
        let name = ps2_mouse::get_device_driver_info().unwrap().name;
        error!("{}: Failed to probe or attach device: {:?}", name, err);
    }

    // clear console input
    if console::clear_input_buf().is_err() {
        error!("Console is locked");
    }

    if let Err(err) = virtio::net::probe_and_attach() {
        let name = virtio::net::get_device_driver_info().unwrap().name;
        error!("{}: Failed to probe or attach device: {:?}", name, err);
    }
}
