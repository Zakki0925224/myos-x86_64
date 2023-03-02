use super::{conf_space::*, msi::MsiCapabilityField};
use alloc::vec::Vec;

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

    pub fn get_device_class(&self) -> (u8, u8, u8)
    {
        let class_code = self.conf_space_header.class_code();
        let subclass_code = self.conf_space_header.subclass();
        let prog_if = self.conf_space_header.prog_if();
        return (class_code, subclass_code, prog_if);
    }

    pub fn is_available_msi_int(&self) -> bool
    {
        return self.conf_space_header.status().caps_list_available();
    }

    pub fn read_caps_list(&self) -> Option<Vec<MsiCapabilityField>>
    {
        if !self.is_available_msi_int()
        {
            return None;
        }

        let mut list = Vec::new();
        let mut caps_ptr = match self.conf_space_header.header_type()
        {
            ConfigurationSpaceHeaderType::NonBridge =>
            {
                self.read_conf_space_non_bridge_field().unwrap().caps_ptr() as usize
            }
            ConfigurationSpaceHeaderType::PciToPciBridge =>
            {
                self.read_conf_space_non_bridge_field().unwrap().caps_ptr() as usize
            }
            _ => return None, // unsupported type
        };

        while caps_ptr != 0
        {
            if let Some(field) =
                MsiCapabilityField::read(self.bus, self.device, self.func, caps_ptr)
            {
                caps_ptr = field.next_ptr() as usize;
                list.push(field);
            }
            else
            {
                break;
            }
        }

        return Some(list);
    }
}
