use super::DescriptorHeader;

#[derive(Debug, Clone)]
#[repr(packed)]
pub struct EndpointDescriptor {
    pub header: DescriptorHeader,
    pub endpoint_addr: u8,
    pub bitmap_attrs: u8,
    pub max_packet_size: u16,
    pub interval: u8,
}

impl EndpointDescriptor {
    pub fn dci(&self) -> usize {
        ((self.endpoint_addr & 0xf) * 2 + (self.endpoint_addr >> 7)) as usize
    }
}
