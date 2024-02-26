use log::{info, warn};

pub mod pci;
pub mod usb;

pub fn init() {
    pci::scan_devices().unwrap();
    info!("pci: Initialized PCI device manager");

    // initialize usb driver
    if let Err(err) = usb::init() {
        warn!("usb: {:?}", err);
    }
    info!("usb: Initialized USB driver");
}
