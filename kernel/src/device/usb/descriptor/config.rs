use modular_bitfield::{bitfield, specifiers::*};

use super::DescriptorHeader;

#[bitfield]
#[derive(Debug)]
#[repr(C)]
pub struct ConfigurationDescriptor
{
    pub header: DescriptorHeader,
    pub total_length: B16,
    pub num_interfaces: B8,
    pub conf_value: B8,
    pub conf_index: B8,
    pub bitmap_attrs: B8,
    pub max_power: B8,
}
