use modular_bitfield::{bitfield, specifiers::*};

use super::DescriptorHeader;

#[bitfield]
#[derive(Debug)]
#[repr(C)]
pub struct ConfigurationDescriptor
{
    header: DescriptorHeader,
    cap_type: B8,
    bcd_version: B16,
    class: B8,
    sub_class: B8,
    protocol: B8,
    conf_count: B8,
}
