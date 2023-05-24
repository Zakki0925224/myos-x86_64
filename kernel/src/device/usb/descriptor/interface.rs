use modular_bitfield::{bitfield, specifiers::*};

use super::DescriptorHeader;

#[bitfield]
#[derive(Debug)]
#[repr(C)]
pub struct InterfaceDescriptor
{
    header: DescriptorHeader,
    interface_num: B8,
    alternate_setting: B8,
    num_of_endpoints: B8,
    class: B8,
    sub_class: B8,
    protocol: B8,
    interface_index: B8,
}
