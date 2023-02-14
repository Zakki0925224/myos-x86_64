use super::conf_space::*;

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

    pub fn get_device_id(&self) -> (u8, u8, u8)
    {
        let class_code = self.conf_space_header.class_code();
        let subclass_code = self.conf_space_header.subclass();
        let prog_if = self.conf_space_header.prog_if();
        return (class_code, subclass_code, prog_if);
    }
}
