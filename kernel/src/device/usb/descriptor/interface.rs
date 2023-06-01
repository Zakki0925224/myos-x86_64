use modular_bitfield::{bitfield, specifiers::*};

use super::DescriptorHeader;

#[bitfield]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct InterfaceDescriptor
{
    pub header: DescriptorHeader,
    pub interface_num: B8,
    pub alternate_setting: B8,
    pub num_of_endpoints: B8,
    pub class: B8,
    pub sub_class: B8,
    pub protocol: B8,
    pub interface_index: B8,
}
