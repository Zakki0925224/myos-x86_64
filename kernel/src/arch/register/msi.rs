use modular_bitfield::{bitfield, specifiers::*, BitfieldSpecifier};

#[bitfield]
#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[repr(C)]
pub struct MsiMessageAddressField
{
    pub xx: B2,
    pub destination_mode: B1,
    pub redirection_hint_indication: B1,
    #[skip]
    reserved: B8,
    pub destination_id: B8,
    const_0xfee: B12,
}

#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[bits = 3]
pub enum DeliveryMode
{
    Fixed = 0,
    LowestPriority = 1,
    Msi = 2,
    Nmi = 4,
    Init = 5,
    ExtInt = 7,
}

#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[bits = 1]
pub enum Level
{
    Deassert = 0,
    Assert = 1,
}

#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[bits = 1]
pub enum TriggerMode
{
    Edge = 0,
    Level = 1,
}

#[bitfield]
#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[repr(C)]
pub struct MsiMessageDataField
{
    pub vector: B8,
    pub delivery_mode: DeliveryMode,
    #[skip]
    reserved: B3,
    pub level: Level,
    pub trigger_mode: TriggerMode,
}
