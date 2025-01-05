use crate::error::Result;

pub mod console;
pub mod local_apic_timer;
pub mod panic_screen;
pub mod pci_bus;
pub mod ps2_keyboard;
pub mod ps2_mouse;
pub mod speaker;
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
    type AttachInput;
    type PollNormalOutput;
    type PollInterruptOutput;

    fn get_device_driver_info(&self) -> Result<DeviceDriverInfo>;
    // check and find device
    fn probe(&mut self) -> Result<()>;
    // initialize device
    fn attach(&mut self, arg: Self::AttachInput) -> Result<()>;
    // normal polling
    fn poll_normal(&mut self) -> Result<Self::PollNormalOutput>;
    // interrupt polling
    fn poll_int(&mut self) -> Result<Self::PollInterruptOutput>;
}
