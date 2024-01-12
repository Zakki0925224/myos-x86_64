use log::{info, warn};

use crate::bus::usb::USB_DRIVER;

pub mod pci;
pub mod usb;

pub fn init() {
    pci::scan_devices().unwrap();
    info!("pci: Initialized PCI device manager");

    // initialize usb driver
    if let Err(err) = USB_DRIVER.try_lock().unwrap().init() {
        warn!("usb: {:?}", err);
    }
}
