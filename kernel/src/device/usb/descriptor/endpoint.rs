use modular_bitfield::{bitfield, specifiers::*};

use super::DescriptorHeader;

#[bitfield]
#[derive(Debug)]
#[repr(C)]
pub struct EndpointDescriptor
{
    header: DescriptorHeader,
    endpoint_addr: B8,
    bitmap_attrs: B8,
    max_packet_size: B16,
    interval: B8,
}
