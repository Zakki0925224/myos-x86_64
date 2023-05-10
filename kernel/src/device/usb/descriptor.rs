#[derive(Debug)]
#[repr(u8)]
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
