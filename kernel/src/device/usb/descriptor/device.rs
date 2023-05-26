use modular_bitfield::{bitfield, specifiers::*};

use super::DescriptorHeader;

#[bitfield]
#[derive(Debug, Clone)]
#[repr(C)]
pub struct DeviceDescriptor
{
    pub header: DescriptorHeader,
    pub bcd_usb_version: B16,
    pub class: B8,
    pub sub_class: B8,
    pub protocol: B8,
    pub max_packet_size: B8,
    pub vendor_id: B16,
    pub product_id: B16,
    pub bcd_device_version: B16,
    pub manufacturer_index: B8,
    pub product_index: B8,
    pub serial_num_index: B8,
    pub num_configs: B8,
}
