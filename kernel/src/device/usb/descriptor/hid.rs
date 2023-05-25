use modular_bitfield::{bitfield, specifiers::*};

use super::*;

#[bitfield]
#[derive(Debug)]
#[repr(C)]
pub struct HumanInterfaceDeviceDescriptor
{
    pub header: DescriptorHeader,
    pub bcd_hid_version: B16,
    pub country_code: B8,
    pub num_descs: B8,
}
