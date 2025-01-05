use super::DescriptorHeader;

#[derive(Debug, Clone, Copy)]
#[repr(packed)]
pub struct HumanInterfaceDeviceDescriptor {
    pub header: DescriptorHeader,
    pub bcd_hid_version: u16,
    pub country_code: u8,
    pub num_descs: u8,
}
