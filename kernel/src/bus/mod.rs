use log::info;

use self::pci::PCI_DEVICE_MAN;

pub mod pci;

pub fn init() {
    PCI_DEVICE_MAN.lock().scan_devices();
    info!("pci: Initialized PCI device manager");
}
