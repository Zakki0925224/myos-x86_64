use crate::{arch::register::msi::{MsiMessageAddressField, MsiMessageDataField}, println};

use super::conf_space::*;
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

    fn read_caps_ptr(&self) -> Option<u8>
    {
        if !self.conf_space_header.status().caps_list_available()
        {
            return None;
        }

        match self.conf_space_header.header_type()
        {
            ConfigurationSpaceHeaderType::NonBridge =>
            {
                return Some(self.read_conf_space_non_bridge_field().unwrap().caps_ptr());
            }
            ConfigurationSpaceHeaderType::PciToPciBridge =>
            {
                return Some(self.read_conf_space_pci_to_pci_bridge_field().unwrap().caps_ptr());
            }
            _ => return None, // unsupported type
        };
    }

    pub fn read_caps_list(&self) -> Option<Vec<MsiCapabilityField>>
    {
        if !self.is_available_msi_int()
        {
            return None;
        }

        let mut list = Vec::new();
        if let Some(caps_ptr) = self.read_caps_ptr()
        {
            let mut caps_ptr = caps_ptr as usize;
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
        }
        else
        {
            return None;
        }

        return Some(list);
    }

    pub fn set_msi_cap(
        &self,
        msg_addr: MsiMessageAddressField,
        msg_data: MsiMessageDataField,
    ) -> Result<(), &'static str>
    {
        if let Some(caps_ptr) = self.read_caps_ptr()
        {
            let mut cap = MsiCapabilityField::new();
            let mut caps_ptr = caps_ptr as usize;
            let caps_list = self.read_caps_list().unwrap();
            let caps_list_len = caps_list.len();

            if caps_list_len == 0
            {
                return Err("MSI capability fields was not found");
            }

            for (i, field) in caps_list.iter().enumerate()
            {
                if field.cap_id() == 5
                {
                    cap = *field;
                    break;
                }

                caps_ptr = field.next_ptr() as usize;

                if i == caps_list_len - 1
                {
                    return Err("MSI capability field was not found");
                }
            }

            let mut msg_ctrl = cap.msg_ctrl();
            msg_ctrl.set_is_enable(true);
            msg_ctrl.set_multiple_msg_enable(0);
            cap.set_msg_ctrl(msg_ctrl);
            cap.set_msg_addr_low(msg_addr);
            cap.set_msg_data(msg_data);

            // write cap
            if let Err(msg) = cap.write(self.bus, self.device, self.func, caps_ptr)
            {
                return Err(msg);
            }
        }
        else
        {
            return Err("Failed to read MSI capability fields");
        }

        return Ok(());
    }
}
