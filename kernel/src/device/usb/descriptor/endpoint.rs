use modular_bitfield::{bitfield, specifiers::*};

use super::DescriptorHeader;

#[bitfield]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct EndpointDescriptor
{
    pub header: DescriptorHeader,
    pub endpoint_addr: B8,
    pub bitmap_attrs: B8,
    pub max_packet_size: B16,
    pub interval: B8,
}

impl EndpointDescriptor
{
    pub fn dci(&self) -> usize
    {
        return ((self.endpoint_addr() & 0xf) * 2 + (self.endpoint_addr() >> 7)) as usize;
    }
}
