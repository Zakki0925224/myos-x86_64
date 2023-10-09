use log::info;

pub mod pci;

pub fn init() {
    loop {
        match pci::scan_devices() {
            Ok(_) => break,
            Err(_) => continue,
        }
    }

    info!("pci: Initialized PCI device manager");
}
