use super::DescriptorHeader;

#[derive(Debug, Clone, Copy)]
#[repr(packed)]
pub struct InterfaceDescriptor {
    pub header: DescriptorHeader,
    pub interface_num: u8,
    pub alternate_setting: u8,
    pub num_of_endpoints: u8,
    pub class: u8,
    pub sub_class: u8,
    pub protocol: u8,
    pub interface_index: u8,
}
