use log::{info, warn};

use crate::bus::usb::USB_DRIVER;

pub mod pci;
pub mod usb;

pub fn init() {
    loop {
        match pci::scan_devices() {
            Ok(_) => break,
            Err(_) => continue,
        }
    }

    info!("pci: Initialized PCI device manager");

    // initialize usb driver
    if let Err(err) = USB_DRIVER.try_lock().unwrap().init() {
        warn!("usb: {:?}", err);
    }
}
