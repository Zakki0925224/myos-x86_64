use super::{DeviceDriverFunction, DeviceDriverInfo};
use crate::{
    error::{Error, Result},
    println,
    util::mutex::Mutex,
};
use alloc::vec::Vec;
use conf_space::*;
use device::{PciDevice, PciDeviceFunctions};
use log::{debug, info};

pub mod conf_space;
mod device;

static mut PCI_BUS_DRIVER: Mutex<PciBusDriver> = Mutex::new(PciBusDriver::new());

struct PciBusDriver {
    device_driver_info: DeviceDriverInfo,
    pci_devices: Vec<PciDevice>,
}

impl PciBusDriver {
    const fn new() -> Self {
        Self {
            device_driver_info: DeviceDriverInfo::new("pci-bus"),
            pci_devices: Vec::new(),
        }
    }

    fn scan_pci_devices(&mut self) {
        let mut devices = Vec::new();

        for bus in 0..PCI_DEVICE_BUS_LEN {
            for device in 0..PCI_DEVICE_DEVICE_LEN {
                for func in 0..PCI_DEVICE_FUNC_LEN {
                    if let Some(pci_device) = PciDevice::try_new(bus, device, func) {
                        debug!(
                            "{}: {}.{}.{} {} found",
                            self.device_driver_info.name,
                            bus,
                            device,
                            func,
                            pci_device
                                .conf_space_header()
                                .get_device_name()
                                .unwrap_or("<UNKNOWN NAME>")
                        );
                        devices.push(pci_device);
                    }
                }
            }
        }

        self.pci_devices = devices;
    }

    fn find_device(&self, bus: usize, device: usize, func: usize) -> Result<&PciDevice> {
        self.pci_devices
            .iter()
            .find(|d| d.bdf() == (bus, device, func))
            .ok_or(Error::Failed("PCI device not found"))
    }

    fn find_device_mut(
        &mut self,
        bus: usize,
        device: usize,
        func: usize,
    ) -> Result<&mut PciDevice> {
        self.pci_devices
            .iter_mut()
            .find(|d| d.bdf() == (bus, device, func))
            .ok_or(Error::Failed("PCI device not found"))
    }

    fn find_devices_by_class_mut(
        &mut self,
        class: u8,
        subclass: u8,
        prog_if: u8,
    ) -> Vec<&mut PciDevice> {
        self.pci_devices
            .iter_mut()
            .filter(|d| d.device_class() == (class, subclass, prog_if))
            .collect()
    }

    fn debug(&self) {
        for d in &self.pci_devices {
            let (bus, device, func) = d.bdf();
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

impl DeviceDriverFunction for PciBusDriver {
    type AttachInput = ();
    type PollNormalOutput = ();
    type PollInterruptOutput = ();

    fn get_device_driver_info(&self) -> Result<DeviceDriverInfo> {
        Ok(self.device_driver_info.clone())
    }

    fn probe(&mut self) -> Result<()> {
        Ok(())
    }

    fn attach(&mut self, _arg: Self::AttachInput) -> Result<()> {
        self.device_driver_info.attached = true;
        Ok(())
    }

    fn poll_normal(&mut self) -> Result<Self::PollNormalOutput> {
        unimplemented!()
    }

    fn poll_int(&mut self) -> Result<Self::PollInterruptOutput> {
        unimplemented!()
    }

    fn read(&mut self) -> Result<Vec<u8>> {
        unimplemented!()
    }

    fn write(&mut self, _data: &[u8]) -> Result<()> {
        unimplemented!()
    }
}

pub fn get_device_driver_info() -> Result<DeviceDriverInfo> {
    unsafe { PCI_BUS_DRIVER.try_lock()? }.get_device_driver_info()
}

pub fn probe_and_attach() -> Result<()> {
    let mut driver = unsafe { PCI_BUS_DRIVER.try_lock() }?;
    let driver_name = driver.get_device_driver_info()?.name;

    driver.probe()?;
    driver.attach(())?;
    info!("{}: Attached!", driver_name);

    info!("{}: Scanning devices...", driver_name);
    driver.scan_pci_devices();
    Ok(())
}

pub fn lspci() -> Result<()> {
    unsafe { PCI_BUS_DRIVER.try_lock() }?.debug();
    Ok(())
}

pub fn is_exist_device(bus: usize, device: usize, func: usize) -> Result<bool> {
    let is_exist = unsafe { PCI_BUS_DRIVER.try_lock() }?
        .find_device(bus, device, func)
        .is_ok();
    Ok(is_exist)
}

pub fn configure_device<F: FnMut(&mut dyn PciDeviceFunctions) -> Result<()>>(
    bus: usize,
    device: usize,
    func: usize,
    mut f: F,
) -> Result<()> {
    let mut driver = unsafe { PCI_BUS_DRIVER.try_lock() }?;
    let device_mut = driver.find_device_mut(bus, device, func)?;

    f(device_mut)
}

pub fn find_devices<F: FnMut(&mut dyn PciDeviceFunctions) -> Result<()>>(
    class: u8,
    subclass: u8,
    prog_if: u8,
    mut f: F,
) -> Result<()> {
    let mut driver = unsafe { PCI_BUS_DRIVER.try_lock() }?;
    let devices = driver.find_devices_by_class_mut(class, subclass, prog_if);

    for device in devices {
        f(device)?;
    }

    Ok(())
}
