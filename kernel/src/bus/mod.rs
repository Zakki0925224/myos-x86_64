use log::{info, warn};

pub mod pci;
pub mod usb;

pub fn init() {
    pci::scan_devices().unwrap();
    info!("pci: PCI device manager initialized");

    // initialize usb driver
    if let Err(err) = usb::init() {
        warn!("usb: {:?}", err);
    }
    info!("usb: USB driver initialized");
}
