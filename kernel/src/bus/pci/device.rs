use alloc::vec::Vec;

use crate::{bus::pci::BaseAddress, println};

use super::*;

#[derive(Debug)]
pub struct PciDevice
{
    pub bus: usize,
    pub device: usize,
    pub func: usize,
    pub conf_space_header: ConfigurationSpaceCommonHeaderField,
}

impl PciDevice
{
    pub fn new(bus: usize, device: usize, func: usize) -> Option<Self>
    {
        if let Some(conf_space_header) =
            ConfigurationSpaceCommonHeaderField::read(bus, device, func)
        {
            return Some(Self { bus, device, func, conf_space_header });
        }
        else
        {
            return None;
        }
    }

    pub fn read_conf_space_non_bridge_field(&self) -> Option<ConfigurationSpaceNonBridgeField>
    {
        match self.conf_space_header.header_type()
        {
            ConfigurationSpaceHeaderType::NonBridge =>
            {
                if let Some(field) =
                    ConfigurationSpaceNonBridgeField::read(self.bus, self.device, self.func)
                {
                    return Some(field);
                }
                else
                {
                    return None;
                }
            }
            _ => return None,
        }
    }

    pub fn read_conf_space_pci_to_pci_bridge_field(
        &self,
    ) -> Option<ConfigurationSpacePciToPciBridgeField>
    {
        match self.conf_space_header.header_type()
        {
            ConfigurationSpaceHeaderType::PciToPciBridge =>
            {
                if let Some(field) =
                    ConfigurationSpacePciToPciBridgeField::read(self.bus, self.device, self.func)
                {
                    return Some(field);
                }
                else
                {
                    return None;
                }
            }
            _ => return None,
        }
    }

    pub fn read_space_pci_to_cardbus_bridge_field(
        &self,
    ) -> Option<ConfigurationSpacePciToCardBusField>
    {
        match self.conf_space_header.header_type()
        {
            ConfigurationSpaceHeaderType::PciToCardBusBridge =>
            {
                if let Some(field) =
                    ConfigurationSpacePciToCardBusField::read(self.bus, self.device, self.func)
                {
                    return Some(field);
                }
                else
                {
                    return None;
                }
            }
            _ => return None,
        }
    }
}

#[derive(Debug)]
pub struct PciDeviceManager
{
    devices: Vec<PciDevice>,
}

impl PciDeviceManager
{
    pub fn new() -> Self
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

        return Self { devices };
    }

    pub fn debug(&self)
    {
        for d in &self.devices
        {
            println!("{}:{}:{}", d.bus, d.device, d.func);
            println!("{:?}", d.conf_space_header.header_type());
            println!("{:?}", d.conf_space_header.get_device_name());
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
