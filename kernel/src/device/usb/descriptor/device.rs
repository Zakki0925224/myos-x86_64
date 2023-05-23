use modular_bitfield::{bitfield, specifiers::*};

use super::DescriptorHeader;

#[bitfield]
#[derive(Debug)]
#[repr(C)]
pub struct DeviceDescriptor
{
    header: DescriptorHeader,
    bcd_usb_version: B16,
    class: B8,
    sub_class: B8,
    protocol: B8,
    max_packet_size: B8,
    vendor_id: B16,
    product_id: B16,
    bcd_device_version: B16,
    manufacturer_index: B8,
    product_index: B8,
    serial_num_index: B8,
    num_configs: B8,
}
