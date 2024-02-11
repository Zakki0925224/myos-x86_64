#[derive(Debug)]
#[repr(u8)]
pub enum RequestTypeRecipient {
    Device = 0,
    Interface = 1,
    Endpoint = 2,
    Other = 3,
}

impl From<u8> for RequestTypeRecipient {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Device,
            1 => Self::Interface,
            2 => Self::Endpoint,
            3 => Self::Other,
            _ => panic!(),
        }
    }
}

#[derive(Debug)]
#[repr(u8)]
pub enum RequestType {
    Standard = 0,
    Class = 1,
    Vendor = 2,
}

impl From<u8> for RequestType {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Standard,
            1 => Self::Class,
            2 => Self::Vendor,
            _ => panic!(),
        }
    }
}

#[derive(Debug)]
#[repr(u8)]
pub enum RequestTypeDirection {
    Out = 0,
    In = 1,
}

impl From<u8> for RequestTypeDirection {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Out,
            1 => Self::In,
            _ => panic!(),
        }
    }
}

#[derive(Debug, Default)]
#[repr(C)]
pub struct SetupRequestType(u8);

impl From<u8> for SetupRequestType {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl SetupRequestType {
    pub fn raw(&self) -> u8 {
        self.0
    }

    pub fn recipient(&self) -> RequestTypeRecipient {
        let value = self.0 & 0x1f;
        RequestTypeRecipient::from(value)
    }

    pub fn set_recipient(&mut self, value: RequestTypeRecipient) {
        let value = value as u8;
        self.0 = (self.0 & !0x1f) | value;
    }

    pub fn ty(&self) -> RequestType {
        let value = (self.0 >> 5) & 0x3;
        RequestType::from(value)
    }

    pub fn set_ty(&mut self, value: RequestType) {
        let value = value as u8;
        self.0 = (self.0 & !0x60) | (value << 2);
    }

    pub fn direction(&self) -> RequestTypeDirection {
        let value = self.0 >> 7;
        RequestTypeDirection::from(value)
    }

    pub fn set_direction(&mut self, value: RequestTypeDirection) {
        let value = value as u8;
        self.0 = (self.0 & !0x80) | (value << 7);
    }
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

impl From<u8> for SetupRequest {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::GetStatus,
            1 => Self::ClearFeature,
            3 => Self::SetFeature,
            5 => Self::SetAddress,
            6 => Self::GetDescriptor,
            7 => Self::SetDescriptor,
            8 => Self::GetConfiguration,
            9 => Self::SetConfiguration,
            10 => Self::GetInterface,
            11 => Self::SetInterface,
            12 => Self::SynchFrame,
            13 => Self::SetEncryption,
            14 => Self::GetEncryption,
            15 => Self::SetHandshake,
            16 => Self::GetHandshake,
            17 => Self::SetConnection,
            18 => Self::SetSecurityData,
            19 => Self::GetSecurityData,
            20 => Self::SetWusbData,
            21 => Self::LoopbackDataWrite,
            22 => Self::LoopbackDataRead,
            23 => Self::SetInterfaceDs,
            26 => Self::GetFwStatus,
            27 => Self::SetFwStatus,
            48 => Self::SetSel,
            49 => Self::SetIsochDelay,
            _ => panic!(),
        }
    }
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

impl From<u8> for TransferType {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::NoDataStage,
            2 => Self::OutDataStage,
            3 => Self::InDataStage,
            _ => panic!(),
        }
    }
}
