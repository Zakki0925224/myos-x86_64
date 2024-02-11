use super::DescriptorHeader;

#[derive(Debug, Clone, Default)]
#[repr(packed)]
pub struct DeviceDescriptor {
    pub header: DescriptorHeader,
    pub bcd_usb_version: u16,
    pub class: u8,
    pub sub_class: u8,
    pub protocol: u8,
    pub max_packet_size: u8,
    pub vendor_id: u16,
    pub product_id: u16,
    pub bcd_device_version: u16,
    pub manufacturer_index: u8,
    pub product_index: u8,
    pub serial_num_index: u8,
    pub num_configs: u8,
}
