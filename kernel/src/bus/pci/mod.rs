use self::{conf_space::*, device::PciDevice};
use crate::println;
use alloc::vec::Vec;
use lazy_static::lazy_static;
use spin::Mutex;

pub mod conf_space;
pub mod device;
pub mod device_id;
pub mod vendor_id;

lazy_static! {
    pub static ref PCI_DEVICE_MAN: Mutex<PciDeviceManager> = Mutex::new(PciDeviceManager::new());
}

#[derive(Debug)]
pub struct PciDeviceManager
{
    devices: Vec<PciDevice>,
}

impl PciDeviceManager
{
    pub fn new() -> Self { return PciDeviceManager { devices: Vec::new() } }

    pub fn scan_devices(&mut self)
    {
        let mut devices = Vec::new();

        for bus in 0..PCI_DEVICE_BUS_LEN
        {
            for device in 0..PCI_DEVICE_DEVICE_LEN
            {
                for func in 0..PCI_DEVICE_FUNC_LEN
                {
                    if let Some(pci_device) = PciDevice::new(bus, device, func)
                    {
                        if pci_device.conf_space_header.is_exist()
                        {
                            devices.push(pci_device);
                        }
                    }
                }
            }
        }

        self.devices = devices;
    }

    pub fn find_by_class(&self, class_code: u8, subclass_code: u8, prog_if: u8) -> Vec<&PciDevice>
    {
        let mut devices = Vec::new();

        let founds = self
            .devices
            .iter()
            .filter(|d| d.get_device_class() == (class_code, subclass_code, prog_if));

        for device in founds
        {
            devices.push(device);
        }

        return devices;
    }

    pub fn find_by_bdf(&self, bus: usize, device: usize, func: usize) -> Option<&PciDevice>
    {
        let found =
            self.devices.iter().find(|d| d.bus == bus && d.device == device && d.func == func);
        return found;
    }

    pub fn debug(&self)
    {
        for d in &self.devices
        {
            println!("{}:{}:{}", d.bus, d.device, d.func);
            println!("{:?}", d.conf_space_header.header_type());
            println!("{:?}", d.conf_space_header.get_device_name());
            println!("{:?}", d.read_caps_list());
            if let Some(field) = d.read_conf_space_non_bridge_field()
            {
                for bar in field.get_bars()
                {
                    let ty = match bar.1
                    {
                        BaseAddress::MemoryAddress32BitSpace(_, _) => "32 bit memory",
                        BaseAddress::MemoryAddress64BitSpace(_, _) => "64 bit memory",
                        BaseAddress::MmioAddressSpace(_) => "I/O",
                    };

                    let addr = match bar.1
                    {
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
