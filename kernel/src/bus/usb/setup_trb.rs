use modular_bitfield::*;

#[derive(BitfieldSpecifier, Debug)]
#[bits = 5]
pub enum RequestTypeRecipient {
    Device = 0,
    Interface = 1,
    Endpoint = 2,
    Other = 3,
}

#[derive(BitfieldSpecifier, Debug)]
#[bits = 2]
pub enum RequestType {
    Standard = 0,
    Class = 1,
    Vendor = 2,
}

#[derive(BitfieldSpecifier, Debug)]
#[bits = 1]
pub enum RequestTypeDirection {
    Out = 0,
    In = 1,
}

#[bitfield]
#[derive(Debug)]
#[repr(C)]
pub struct SetupRequestType {
    pub recipient: RequestTypeRecipient,
    pub ty: RequestType,
    pub direction: RequestTypeDirection,
}

#[derive(Debug)]
#[repr(u8)]
pub enum SetupRequest {
    GetStatus = 0,
    ClearFeature = 1,
    SetFeature = 3,
    SetAddress = 5,
    GetDescriptor = 6,
    SetDescriptor = 7,
    GetConfiguration = 8,
    SetConfiguration = 9,
    GetInterface = 10,
    SetInterface = 11,
    SynchFrame = 12,
    SetEncryption = 13,
    GetEncryption = 14,
    SetHandshake = 15,
    GetHandshake = 16,
    SetConnection = 17,
    SetSecurityData = 18,
    GetSecurityData = 19,
    SetWusbData = 20,
    LoopbackDataWrite = 21,
    LoopbackDataRead = 22,
    SetInterfaceDs = 23,
    GetFwStatus = 26,
    SetFwStatus = 27,
    SetSel = 48,
    SetIsochDelay = 49,
}

impl SetupRequest {
    pub const GET_REPORT: Self = Self::ClearFeature;
    pub const SET_PROTOCOL: Self = Self::SetInterface;
}

#[derive(Debug)]
#[repr(u8)]
pub enum TransferType {
    NoDataStage = 0,
    OutDataStage = 2,
    InDataStage = 3,
}
