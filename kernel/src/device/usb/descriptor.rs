use modular_bitfield::{bitfield, specifiers::*, BitfieldSpecifier};

#[derive(BitfieldSpecifier, Debug)]
#[bits = 8]
pub enum DescriptorType
{
    Device = 1,
    Configration = 2,
    String = 3,
    Interface = 4,
    Endpoint = 5,
    InterfacePower = 8,
    Otg = 9,
    Debug = 10,
    InterfaceAssociation = 11,
    Bos = 15,
    DeviceCapability = 16,
    SuperspeedUsbEndpointCompanion = 48,
    SuperspeedIsochronousEndpointCompanion = 49,
}

#[bitfield]
#[derive(BitfieldSpecifier, Debug)]
#[repr(C)]
pub struct DescriptorHeader
{
    length: B8, // bytes
    ty: DescriptorType,
}

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
