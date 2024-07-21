use self::{
    conf_space::*,
    device::{PciDevice, PciDeviceFunctions},
};
use crate::{
    error::{Error, Result},
    println,
    util::mutex::Mutex,
};
use alloc::vec::Vec;

pub mod conf_space;
pub mod device;
pub mod vendor_id;

static mut PCI_DEVICE_MAN: Mutex<PciDeviceManager> = Mutex::new(PciDeviceManager::new());

#[derive(Debug)]
pub struct PciDeviceManager {
    devices: Vec<PciDevice>,
}

impl PciDeviceManager {
    pub const fn new() -> Self {
        Self {
            devices: Vec::new(),
        }
    }

    pub fn scan_devices(&mut self) {
        let mut devices = Vec::new();

        for bus in 0..PCI_DEVICE_BUS_LEN {
            for device in 0..PCI_DEVICE_DEVICE_LEN {
                for func in 0..PCI_DEVICE_FUNC_LEN {
                    let pci_device = match PciDevice::new(bus, device, func) {
                        Some(d) => d,
                        None => continue,
                    };

                    devices.push(pci_device);
                }
            }
        }

        self.devices = devices;
    }

    pub fn find_device(&self, bus: usize, device: usize, func: usize) -> Result<&PciDevice> {
        match self
            .devices
            .iter()
            .find(|d| d.device_bdf() == (bus, device, func))
        {
            Some(d) => Ok(d),
            None => Err(Error::Failed("PCI device was not found").into()),
        }
    }

    pub fn find_device_mut(
        &mut self,
        bus: usize,
        device: usize,
        func: usize,
    ) -> Result<&mut PciDevice> {
        match self
            .devices
            .iter_mut()
            .find(|d| d.device_bdf() == (bus, device, func))
        {
            Some(d) => Ok(d),
            None => Err(Error::Failed("PCI device was not found").into()),
        }
    }

    pub fn find_device_by_class_mut(
        &mut self,
        class_code: u8,
        subclass_code: u8,
        prog_if: u8,
    ) -> Vec<&mut PciDevice> {
        self.devices
            .iter_mut()
            .filter(|d| d.device_class() == (class_code, subclass_code, prog_if))
            .collect()
    }

    pub fn debug(&self) {
        for d in &self.devices {
            let (bus, device, func) = d.device_bdf();
            let conf_space_header = d.conf_space_header();
            println!("{}:{}:{}", bus, device, func);
            println!("{:?}", conf_space_header.get_header_type());
            println!("{:?}", conf_space_header.get_device_name());
            println!(
                "vendor: 0x{:x}, device: 0x{:x}",
                conf_space_header.vendor_id, conf_space_header.device_id
            );
            println!(
                "class: {}, subclass: {}, if: {}\n",
                conf_space_header.class_code, conf_space_header.subclass, conf_space_header.prog_if
            );
            if let Ok(field) = d.read_conf_space_non_bridge_field() {
                for bar in field.get_bars().unwrap() {
                    let ty = match bar.1 {
                        BaseAddress::MemoryAddress32BitSpace(_, _) => "32 bit memory",
                        BaseAddress::MemoryAddress64BitSpace(_, _) => "64 bit memory",
                        BaseAddress::MmioAddressSpace(_) => "I/O",
                    };

                    let addr = match bar.1 {
                        BaseAddress::MemoryAddress32BitSpace(addr, _) => addr.get() as usize,
                        BaseAddress::MemoryAddress64BitSpace(addr, _) => addr.get() as usize,
                        BaseAddress::MmioAddressSpace(addr) => addr as usize,
                    };

                    println!("BAR{}: {} at 0x{:x}", bar.0, ty, addr);
                }
            }
            println!("--------------");
        }
    }
}

pub fn scan_devices() -> Result<()> {
    unsafe { PCI_DEVICE_MAN.try_lock() }?.scan_devices();
    Ok(())
}

pub fn lspci() -> Result<()> {
    unsafe { PCI_DEVICE_MAN.try_lock() }?.debug();
    Ok(())
}

pub fn is_exit_device(bus: usize, device: usize, func: usize) -> Result<bool> {
    return Ok(unsafe { PCI_DEVICE_MAN.try_lock() }?
        .find_device(bus, device, func)
        .is_ok());
}

pub fn configure_device<F: FnMut(&mut dyn PciDeviceFunctions) -> Result<()>>(
    bus: usize,
    device: usize,
    func: usize,
    mut f: F,
) -> Result<()> {
    return f(unsafe { PCI_DEVICE_MAN.try_lock() }?.find_device_mut(bus, device, func)?);
}

pub fn find_devices<F: FnMut(&mut dyn PciDeviceFunctions) -> Result<()>>(
    class_code: u8,
    subclass_code: u8,
    prog_if: u8,
    mut f: F,
) -> Result<()> {
    let mut pci_device_man = unsafe { PCI_DEVICE_MAN.try_lock() }?;
    let devices = pci_device_man.find_device_by_class_mut(class_code, subclass_code, prog_if);
    for d in devices {
        f(d)?;
    }
    Ok(())
}
