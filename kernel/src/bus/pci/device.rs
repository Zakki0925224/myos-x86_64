use super::conf_space::{self, *};
use crate::{
    arch::register::msi::{MsiMessageAddressField, MsiMessageDataField},
    error::{Error, Result},
};
use alloc::vec::Vec;

pub trait PciDeviceFunctions {
    fn conf_space_header(&self) -> &ConfigurationSpaceCommonHeaderField;
    fn read_conf_space_non_bridge_field(&self) -> Result<ConfigurationSpaceNonBridgeField>;
    fn read_conf_space_pci_to_pci_bridge_field(
        &self,
    ) -> Result<ConfigurationSpacePciToPciBridgeField>;
    fn read_space_pci_to_cardbus_bridge_field(&self)
        -> Result<ConfigurationSpacePciToCardBusField>;
    fn read_interrupt_line(&self) -> Result<u8>;
    fn write_interrupt_line(&self, value: u8) -> Result<()>;
    fn device_class(&self) -> (u8, u8, u8);
    fn device_bdf(&self) -> (usize, usize, usize);
    fn is_available_msi_int(&self) -> bool;
    fn read_msi_caps_list(&self) -> Vec<MsiCapabilityField>;
    fn set_msi_cap(
        &self,
        msg_addr: MsiMessageAddressField,
        msg_data: MsiMessageDataField,
    ) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct PciDevice {
    bus: usize,
    device: usize,
    func: usize,
    conf_space_header: ConfigurationSpaceCommonHeaderField,
}

impl PciDevice {
    pub fn new(bus: usize, device: usize, func: usize) -> Option<Self> {
        let conf_space_header = match ConfigurationSpaceCommonHeaderField::read(bus, device, func) {
            Ok(header) => header,
            Err(_) => return None,
        };
        if !conf_space_header.is_exist() {
            return None;
        }

        Some(Self {
            bus,
            device,
            func,
            conf_space_header,
        })
    }

    fn read_caps_ptr(&self) -> Option<u8> {
        if !self.conf_space_header.status.caps_list_available() {
            return None;
        }

        match self.conf_space_header.get_header_type() {
            ConfigurationSpaceHeaderType::NonBridge
            | ConfigurationSpaceHeaderType::MultiFunction => {
                Some(self.read_conf_space_non_bridge_field().unwrap().caps_ptr)
            }
            ConfigurationSpaceHeaderType::PciToPciBridge => Some(
                self.read_conf_space_pci_to_pci_bridge_field()
                    .unwrap()
                    .caps_ptr,
            ),
            _ => None, // unsupported type
        }
    }
}

impl PciDeviceFunctions for PciDevice {
    fn conf_space_header(&self) -> &ConfigurationSpaceCommonHeaderField {
        &self.conf_space_header
    }

    fn read_conf_space_non_bridge_field(&self) -> Result<ConfigurationSpaceNonBridgeField> {
        match self.conf_space_header.get_header_type() {
            ConfigurationSpaceHeaderType::NonBridge
            | ConfigurationSpaceHeaderType::MultiFunction => {
                ConfigurationSpaceNonBridgeField::read(self.bus, self.device, self.func)
            }
            _ => Err(Error::Failed("Invalid configuration space header type")),
        }
    }

    fn read_conf_space_pci_to_pci_bridge_field(
        &self,
    ) -> Result<ConfigurationSpacePciToPciBridgeField> {
        match self.conf_space_header.get_header_type() {
            ConfigurationSpaceHeaderType::PciToPciBridge => {
                ConfigurationSpacePciToPciBridgeField::read(self.bus, self.device, self.func)
            }
            _ => Err(Error::Failed("Invalid configuration space header type")),
        }
    }

    fn read_space_pci_to_cardbus_bridge_field(
        &self,
    ) -> Result<ConfigurationSpacePciToCardBusField> {
        match self.conf_space_header.get_header_type() {
            ConfigurationSpaceHeaderType::PciToCardBusBridge => {
                ConfigurationSpacePciToCardBusField::read(self.bus, self.device, self.func)
            }
            _ => Err(Error::Failed("Invalid configuration space header type")),
        }
    }

    fn read_interrupt_line(&self) -> Result<u8> {
        let data = conf_space::read_conf_space(self.bus, self.device, self.func, 0x3c)?;
        Ok(data as u8)
    }

    fn write_interrupt_line(&self, value: u8) -> Result<()> {
        let data = conf_space::read_conf_space(self.bus, self.device, self.func, 0x3c)? & !0xff
            | value as u32;
        conf_space::write_conf_space(self.bus, self.device, self.func, 0x3c, data)?;

        Ok(())
    }

    fn device_class(&self) -> (u8, u8, u8) {
        let class_code = self.conf_space_header.class_code;
        let subclass_code = self.conf_space_header.subclass;
        let prog_if = self.conf_space_header.prog_if;
        (class_code, subclass_code, prog_if)
    }

    fn device_bdf(&self) -> (usize, usize, usize) {
        (self.bus, self.device, self.func)
    }

    fn is_available_msi_int(&self) -> bool {
        self.conf_space_header.status.caps_list_available()
    }

    fn read_msi_caps_list(&self) -> Vec<MsiCapabilityField> {
        let mut list = Vec::new();

        if !self.is_available_msi_int() {
            return list;
        }

        if let Some(caps_ptr) = self.read_caps_ptr() {
            let mut caps_ptr = caps_ptr as usize;
            while caps_ptr != 0 {
                if let Ok(field) =
                    MsiCapabilityField::read(self.bus, self.device, self.func, caps_ptr)
                {
                    caps_ptr = field.next_ptr as usize;
                    list.push(field);
                } else {
                    break;
                }
            }
        }

        list
    }

    fn set_msi_cap(
        &self,
        msg_addr: MsiMessageAddressField,
        msg_data: MsiMessageDataField,
    ) -> Result<()> {
        if let Some(caps_ptr) = self.read_caps_ptr() {
            let mut cap = MsiCapabilityField::default();
            let mut caps_ptr = caps_ptr as usize;
            let caps_list = self.read_msi_caps_list();
            let caps_list_len = caps_list.len();

            if caps_list_len == 0 {
                return Err(Error::Failed("MSI capability fields was not found"));
            }

            for (i, field) in caps_list.iter().enumerate() {
                if field.cap_id == 5 {
                    cap = *field;
                    break;
                }

                caps_ptr = field.next_ptr as usize;

                if i == caps_list_len - 1 {
                    return Err(Error::Failed("MSI capability fields was not found"));
                }
            }

            let mut msg_ctrl = cap.msg_ctrl;
            msg_ctrl.set_is_enable(true);
            msg_ctrl.set_multiple_msg_enable(0);
            cap.msg_ctrl = msg_ctrl;
            cap.msg_addr_low = msg_addr;
            cap.msg_data = msg_data;

            // write cap
            cap.write(self.bus, self.device, self.func, caps_ptr)?;
        } else {
            return Err(Error::Failed("Failed to read MSI capability fields"));
        }

        Ok(())
    }
}
