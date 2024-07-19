use log::{error, info};

pub mod console;
pub mod manager;
pub mod ps2_keyboard;
pub mod ps2_mouse;
pub mod serial;
pub mod usb;

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub struct DeviceId(usize);

// impl DeviceId {
//     pub fn new() -> Self {
//         static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
//         Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
//     }

//     pub fn get(&self) -> usize {
//         self.0
//     }
// }

// #[derive(Debug, Clone, Copy)]
// pub enum DeviceClass {
//     Test,
// }

// #[derive(Debug, Clone, Copy, PartialEq)]
// pub enum DevicePollMode {
//     //Interrupt,
//     Sync,
//     //Async,
//     None,
// }

// #[derive(Debug, Clone)]
// pub struct DeviceDriverInfo {
//     pub id: DeviceId,
//     pub class: DeviceClass,
//     pub name: &'static str,
//     pub device_file_path: Option<&'static str>,
// }

// pub trait DeviceFunction {
//     fn get_info(&self) -> DeviceDriverInfo;
//     fn is_attached(&self) -> bool;
//     fn poll_mode(&self) -> DevicePollMode;
//     fn attach(&mut self) -> Result<()>;
//     fn detach(&mut self) -> Result<()>;
//     fn update_poll(&mut self) -> Result<()>;
// }

// pub struct DeviceDriverBase {
//     info: DeviceDriverInfo,
//     attached: bool,
//     poll_mode: DevicePollMode,
// }

// impl DeviceDriverBase {
//     pub fn new(
//         class: DeviceClass,
//         name: &'static str,
//         device_file_path: Option<&'static str>,
//         poll_mode: DevicePollMode,
//     ) -> Self {
//         Self {
//             info: DeviceDriverInfo {
//                 id: DeviceId::new(),
//                 class,
//                 name,
//                 device_file_path,
//             },
//             attached: false,
//             poll_mode,
//         }
//     }
// }

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

    // manager::register_device(Box::new(TestDevice::new())).unwrap();
    // manager::register_device(Box::new(TestDevice::new())).unwrap();
    // manager::register_device(Box::new(TestDevice::new())).unwrap();
    // manager::attach_all_devices().unwrap();
}
