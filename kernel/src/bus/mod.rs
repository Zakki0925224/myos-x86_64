use log::{info, warn};

pub mod pci;
pub mod usb;

pub fn init() {
    info!("pci: Scanning all PCI devices...");
    pci::scan_devices().unwrap();
    info!("pci: All PCI devices registered");

    // initialize usb driver
    if let Err(err) = usb::init() {
        warn!("usb: {:?}", err);
    }
    info!("usb: USB driver initialized");
}
