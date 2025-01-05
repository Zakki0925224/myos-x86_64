use super::DescriptorHeader;

#[derive(Debug, Clone, Copy)]
#[repr(packed)]
pub struct ConfigurationDescriptor {
    pub header: DescriptorHeader,
    pub total_length: u16,
    pub num_interfaces: u8,
    pub conf_value: u8,
    pub conf_index: u8,
    pub bitmap_attrs: u8,
    pub max_power: u8,
}
