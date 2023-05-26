use alloc::vec::Vec;
use modular_bitfield::{bitfield, specifiers::*, BitfieldSpecifier};

use self::{config::ConfigurationDescriptor, device::DeviceDescriptor, endpoint::EndpointDescriptor, hid::HumanInterfaceDeviceDescriptor, interface::InterfaceDescriptor};

pub mod config;
pub mod device;
pub mod endpoint;
pub mod hid;
pub mod interface;

#[derive(Debug, Clone)]
pub enum Descriptor
{
    Device(DeviceDescriptor),
    Configuration(ConfigurationDescriptor),
    Endpoint(EndpointDescriptor),
    Interface(InterfaceDescriptor),
    HumanInterfaceDevice(HumanInterfaceDeviceDescriptor, Vec<DescriptorHeader>),
    Unsupported(DescriptorType),
}

#[derive(BitfieldSpecifier, Debug, Clone)]
#[bits = 8]
pub enum DescriptorType
{
    Device = 0x1,
    Configration = 0x2,
    String = 0x3,
    Interface = 0x4,
    Endpoint = 0x5,
    DeviceQualifier = 0x6,
    OtherSpeedConfiguration = 0x7,
    InterfacePower = 0x8,
    Otg = 0x9,
    Debug = 0xa,
    InterfaceAssociation = 0xb,
    BinaryDeviceObjectStore = 0xf,
    DeviceCapability = 0x10,
    SuperspeedUsbEndpointCompanion = 0x30,
    SuperspeedIsochronousEndpointCompanion = 0x31,

    // HID
    HumanInterfaceDevice = 0x21,
    Report = 0x22,
    Physical = 0x23,

    UsbHub = 0x29,
}

#[bitfield]
#[derive(BitfieldSpecifier, Debug, Clone)]
#[repr(C)]
pub struct DescriptorHeader
{
    pub length: B8, // bytes
    pub ty: DescriptorType,
}
