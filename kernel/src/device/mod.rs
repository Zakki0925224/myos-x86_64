use crate::error::Result;
use alloc::string::String;
use core::sync::atomic::{AtomicUsize, Ordering};
use log::{error, info};

pub mod console;
pub mod ps2_keyboard;
pub mod ps2_mouse;
pub mod serial;
mod test;
pub mod usb;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceId(usize);

impl DeviceId {
    pub fn new() -> Self {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }

    pub fn get(&self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DeviceClass {
    Test,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DevicePollMode {
    //Interrupt,
    Sync,
    Async,
    None,
}

#[derive(Debug, Clone)]
pub struct DeviceDriverInfo {
    pub id: DeviceId,
    pub class: DeviceClass,
    pub name: String,
    pub device_file_path: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeviceDriverError {
    InvalidPollMode(DevicePollMode),
}

trait DeviceFunction {
    fn new() -> Self;
    fn get_info(&self) -> DeviceDriverInfo;
    fn attach(&mut self) -> Result<()>;
    fn detach(&mut self) -> Result<()>;
    fn update_poll(&mut self) -> Result<()>;
    async fn update_poll_async(&mut self) -> Result<()>;
}

struct DeviceDriverBase {
    info: DeviceDriverInfo,
    attached: bool,
    poll_mode: DevicePollMode,
}

impl DeviceDriverBase {
    pub fn new(
        class: DeviceClass,
        name: String,
        device_file_path: Option<String>,
        poll_mode: DevicePollMode,
    ) -> Self {
        Self {
            info: DeviceDriverInfo {
                id: DeviceId::new(),
                class,
                name,
                device_file_path,
            },
            attached: false,
            poll_mode,
        }
    }
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
}
