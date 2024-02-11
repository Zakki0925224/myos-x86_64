use crate::bus::usb::setup_trb::*;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub enum TransferRequestBlockType {
    Invalid = 0,
    Normal = 1,
    SetupStage = 2,
    DataStage = 3,
    StatusStage = 4,
    Isoch = 5,
    Link = 6,
    EventData = 7,
    NoOp = 8,
    EnableSlotCommand = 9,
    DisableSlotCommand = 10,
    AddressDeviceCommand = 11,
    ConfigureEndpointCommnad = 12,
    EvaluateContextCommand = 13,
    ResetEndpointCommand = 14,
    StopEndpointCommand = 15,
    SetTrDequeuePointerCommand = 16,
    ResetDeviceCommand = 17,
    ForceEventCommand = 18,
    NegotiateBandwidthCommand = 19,
    SetLatencyToleranceValueCommand = 20,
    GetPortBandWithCommand = 21,
    ForceHeaderCommand = 22,
    NoOpCommand = 23,
    GetExtendedPropertyCommand = 24,
    SetExtendedPropertyCommand = 25,
    TransferEvent = 32,
    CommandCompletionEvent = 33,
    PortStatusChangeEvent = 34,
    BandwithRequestEvent = 35,
    DoorbellEvent = 36,
    HostControllerEvent = 37,
    DeviceNotificationEvent = 38,
    MfIndexWrapEvent = 39,
    Reserved,
}

impl From<u8> for TransferRequestBlockType {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Invalid,
            1 => Self::Normal,
            2 => Self::SetupStage,
            3 => Self::DataStage,
            4 => Self::StatusStage,
            5 => Self::Isoch,
            6 => Self::Link,
            7 => Self::EventData,
            8 => Self::NoOp,
            9 => Self::EnableSlotCommand,
            10 => Self::DisableSlotCommand,
            11 => Self::AddressDeviceCommand,
            12 => Self::ConfigureEndpointCommnad,
            13 => Self::EvaluateContextCommand,
            14 => Self::ResetEndpointCommand,
            15 => Self::StopEndpointCommand,
            16 => Self::SetTrDequeuePointerCommand,
            17 => Self::ResetDeviceCommand,
            18 => Self::ForceEventCommand,
            19 => Self::NegotiateBandwidthCommand,
            20 => Self::SetLatencyToleranceValueCommand,
            21 => Self::GetPortBandWithCommand,
            22 => Self::ForceHeaderCommand,
            23 => Self::NoOpCommand,
            24 => Self::GetExtendedPropertyCommand,
            25 => Self::SetExtendedPropertyCommand,
            32 => Self::TransferEvent,
            33 => Self::CommandCompletionEvent,
            34 => Self::PortStatusChangeEvent,
            35 => Self::BandwithRequestEvent,
            36 => Self::DoorbellEvent,
            37 => Self::HostControllerEvent,
            38 => Self::DeviceNotificationEvent,
            39 => Self::MfIndexWrapEvent,
            _ => Self::Reserved,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CompletionCode {
    Invalid = 0,
    Success = 1,
    DataBufferError = 2,
    BabbleDetectedError = 3,
    UsbTransactionError = 4,
    TrbError = 5,
    StallError = 6,
    ResourceError = 7,
    BandwidthError = 8,
    NoSlotsAvailableError = 9,
    InvalidStreamTypeError = 10,
    SlotNotEnabledError = 11,
    EndpointNotEnabledError = 12,
    ShortPacket = 13,
    RingUnderrun = 14,
    RingOverrun = 15,
    VfEventRingFullError = 16,
    ParameterError = 17,
    BandwithOverrunError = 18,
    ContextStateError = 19,
    NoPingResponseError = 20,
    EventRingFullError = 21,
    IncompatibleDeviceError = 22,
    MissedServiceError = 23,
    CommandRingStopped = 24,
    CommandAbortred = 25,
    Stopped = 26,
    StuppedBecauseLengthInvalid = 27,
    StoppedBecauseShortPacket = 28,
    MaxExitLatencyTooLargeError = 29,
    IsochBufferOverrun = 31,
    EventLostError = 32,
    UndefinedError = 33,
    InvalidStreamIdError = 34,
    SecondaryBandwithError = 35,
    SplitTransactionError = 36,
    Reserved,
}

impl From<u8> for CompletionCode {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Invalid,
            1 => Self::Success,
            2 => Self::DataBufferError,
            3 => Self::BabbleDetectedError,
            4 => Self::UsbTransactionError,
            5 => Self::TrbError,
            6 => Self::StallError,
            7 => Self::ResourceError,
            8 => Self::BandwidthError,
            9 => Self::NoSlotsAvailableError,
            10 => Self::InvalidStreamTypeError,
            11 => Self::SlotNotEnabledError,
            12 => Self::EndpointNotEnabledError,
            13 => Self::ShortPacket,
            14 => Self::RingUnderrun,
            15 => Self::RingOverrun,
            16 => Self::VfEventRingFullError,
            17 => Self::ParameterError,
            18 => Self::BandwithOverrunError,
            19 => Self::ContextStateError,
            20 => Self::NoPingResponseError,
            21 => Self::EventRingFullError,
            22 => Self::IncompatibleDeviceError,
            23 => Self::MissedServiceError,
            24 => Self::CommandRingStopped,
            25 => Self::CommandAbortred,
            26 => Self::Stopped,
            27 => Self::StuppedBecauseLengthInvalid,
            28 => Self::StoppedBecauseShortPacket,
            29 => Self::MaxExitLatencyTooLargeError,
            31 => Self::IsochBufferOverrun,
            32 => Self::EventLostError,
            33 => Self::UndefinedError,
            34 => Self::InvalidStreamIdError,
            35 => Self::SecondaryBandwithError,
            36 => Self::SplitTransactionError,
            _ => Self::Reserved,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct TransferRequestBlock {
    pub param: u64,
    pub status: u32,
    flags: u16,
    pub ctrl_regs: u16,
}

impl TransferRequestBlock {
    // Link TRB
    pub fn toggle_cycle(&self) -> Option<bool> {
        if self.trb_type() != TransferRequestBlockType::Link {
            return None;
        }

        let flags = self.other_flags();

        Some((flags & 0x1) != 0)
    }

    pub fn set_toggle_cycle(&mut self, new_val: bool) {
        if self.trb_type() != TransferRequestBlockType::Link {
            return;
        }

        let toggle_cycle = if new_val { 1 } else { 0 };
        let flags = (self.other_flags() & !0x1) | toggle_cycle;
        self.set_other_flags(flags);
    }

    // Command Completion Event TRB
    pub fn slot_id(&self) -> Option<usize> {
        let slot_id = (self.ctrl_regs >> 8) as usize;

        if self.trb_type() != TransferRequestBlockType::CommandCompletionEvent
            && self.trb_type() != TransferRequestBlockType::TransferEvent
        {
            return None;
        }

        if slot_id == 0 {
            return None;
        }

        Some(slot_id)
    }

    // Port Status Change Event TRB
    pub fn port_id(&self) -> Option<usize> {
        if self.trb_type() != TransferRequestBlockType::PortStatusChangeEvent {
            return None;
        }

        Some((self.param >> 24) as usize)
    }

    // Setup Stage TRB
    pub fn set_transfer_type(&mut self, new_val: TransferType) {
        if self.trb_type() != TransferRequestBlockType::SetupStage {
            return;
        }

        let ctrl_regs = (self.ctrl_regs & !0x3) | new_val as u8 as u16;
        self.ctrl_regs = ctrl_regs;
    }

    pub fn transfer_type(&self) -> Option<TransferType> {
        if self.trb_type() != TransferRequestBlockType::SetupStage {
            return None;
        }

        Some(TransferType::from((self.ctrl_regs & 0x3) as u8))
    }

    pub fn set_setup_request_type(&mut self, new_val: SetupRequestType) {
        if self.trb_type() != TransferRequestBlockType::SetupStage {
            return;
        }

        let param = (self.param & !0xff) | new_val.raw() as u64;
        self.param = param;
    }

    pub fn setup_request_type(&self) -> Option<SetupRequestType> {
        if self.trb_type() != TransferRequestBlockType::SetupStage {
            return None;
        }

        Some(SetupRequestType::from(self.param as u8))
    }

    pub fn set_setup_request(&mut self, new_val: SetupRequest) {
        if self.trb_type() != TransferRequestBlockType::SetupStage {
            return;
        }

        let param = (self.param & !0xff00) | ((new_val as u64) << 8);
        self.param = param;
    }

    pub fn setup_request(&self) -> Option<SetupRequest> {
        if self.trb_type() != TransferRequestBlockType::SetupStage {
            return None;
        }

        Some(SetupRequest::from((self.param >> 8) as u8))
    }

    pub fn set_setup_value(&mut self, new_val: u16) {
        if self.trb_type() != TransferRequestBlockType::SetupStage {
            return;
        }

        let param = (self.param & !0xffff0000) | ((new_val as u64) << 16);
        self.param = param;
    }

    pub fn setup_value(&self) -> Option<u16> {
        if self.trb_type() != TransferRequestBlockType::SetupStage {
            return None;
        }

        Some((self.param >> 16) as u16)
    }

    pub fn set_setup_index(&mut self, new_val: u16) {
        if self.trb_type() != TransferRequestBlockType::SetupStage {
            return;
        }

        let param = (self.param & !0xffff00000000) | ((new_val as u64) << 32);
        self.param = param;
    }

    pub fn setup_index(&self) -> Option<u16> {
        if self.trb_type() != TransferRequestBlockType::SetupStage {
            return None;
        }

        Some((self.param >> 32) as u16)
    }

    pub fn set_setup_length(&mut self, new_val: u16) {
        if self.trb_type() != TransferRequestBlockType::SetupStage {
            return;
        }

        let param = (self.param & !0xffff000000000000) | ((new_val as u64) << 48);
        self.param = param;
    }

    pub fn setup_length(&self) -> Option<u16> {
        if self.trb_type() != TransferRequestBlockType::SetupStage {
            return None;
        }

        Some((self.param >> 48) as u16)
    }

    pub fn completion_code(&self) -> Option<CompletionCode> {
        match self.trb_type() {
            TransferRequestBlockType::TransferEvent => (),
            TransferRequestBlockType::CommandCompletionEvent => (),
            TransferRequestBlockType::PortStatusChangeEvent => (),
            TransferRequestBlockType::BandwithRequestEvent => (),
            TransferRequestBlockType::DoorbellEvent => (),
            TransferRequestBlockType::HostControllerEvent => (),
            TransferRequestBlockType::DeviceNotificationEvent => (),
            TransferRequestBlockType::MfIndexWrapEvent => (),
            _ => return None,
        }

        Some(CompletionCode::from((self.status >> 24) as u8))
    }

    // transfer event TRB
    pub fn trb_transfer_length(&self) -> Option<usize> {
        if self.trb_type() != TransferRequestBlockType::TransferEvent {
            return None;
        }

        Some((self.status & 0xfff) as usize)
    }

    pub fn endpoint_id(&self) -> Option<usize> {
        if self.trb_type() != TransferRequestBlockType::TransferEvent {
            return None;
        }

        Some((self.ctrl_regs & 0x1f) as usize)
    }

    pub fn cycle_bit(&self) -> bool {
        (self.flags & 0x1) != 0
    }

    pub fn set_cycle_bit(&mut self, value: bool) {
        let value = if value { 0x1 } else { 0x0 };
        self.flags = (self.flags & !0x1) | value;
    }

    pub fn trb_type(&self) -> TransferRequestBlockType {
        let value = ((self.flags >> 10) as u8) & 0x3f;
        TransferRequestBlockType::from(value)
    }

    pub fn set_trb_type(&mut self, value: TransferRequestBlockType) {
        let value = value as u8;
        self.flags = (self.flags & !0xfc00) | ((value as u16) << 10);
    }

    pub fn other_flags(&self) -> u16 {
        (self.flags >> 1) & 0x1ff
    }

    pub fn set_other_flags(&mut self, value: u16) {
        let value = value & 0x1ff;
        self.flags = (self.flags & !0x3fe) | (value << 1);
    }
}
