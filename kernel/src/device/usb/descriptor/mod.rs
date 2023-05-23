use modular_bitfield::{bitfield, specifiers::*, BitfieldSpecifier};

pub mod config;
pub mod device;

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
