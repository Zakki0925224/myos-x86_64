use core::mem::transmute;

use modular_bitfield::{bitfield, specifiers::*, BitfieldSpecifier};

use crate::device::usb::setup_trb::*;

#[derive(BitfieldSpecifier, Debug, Clone, Copy, Eq, PartialEq)]
#[bits = 6]
pub enum TransferRequestBlockType
{
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

#[derive(BitfieldSpecifier, Debug, Clone, Copy, Eq, PartialEq)]
#[bits = 8]
pub enum CompletionCode
{
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

#[bitfield]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct TransferRequestBlock
{
    pub param: B64,
    pub status: B32,
    pub cycle_bit: bool,
    pub other_flags: B9,
    pub trb_type: TransferRequestBlockType,
    pub ctrl_regs: B16,
}

impl TransferRequestBlock
{
    // Link TRB
    pub fn toggle_cycle(&self) -> Option<bool>
    {
        if self.trb_type() != TransferRequestBlockType::Link
        {
            return None;
        }

        let flags = self.other_flags();

        return Some((flags & 0x1) != 0);
    }

    pub fn set_toggle_cycle(&mut self, new_val: bool)
    {
        if self.trb_type() != TransferRequestBlockType::Link
        {
            return;
        }

        let toggle_cycle = if new_val { 1 } else { 0 };
        let flags = (self.other_flags() & !0x1) | toggle_cycle;
        self.set_other_flags(flags);
    }

    // Command Completion Event TRB
    pub fn slot_id(&self) -> Option<usize>
    {
        let slot_id = (self.ctrl_regs() >> 8) as usize;

        if self.trb_type() != TransferRequestBlockType::CommandCompletionEvent
        {
            return None;
        }

        if slot_id == 0
        {
            return None;
        }

        return Some(slot_id);
    }

    // Port Status Change Event TRB
    pub fn port_id(&self) -> Option<usize>
    {
        if self.trb_type() != TransferRequestBlockType::PortStatusChangeEvent
        {
            return None;
        }

        return Some((self.param() >> 24) as usize);
    }

    // Setup Stage TRB
    pub fn set_setup_request_type(&mut self, new_val: SetupRequestType)
    {
        if self.trb_type() != TransferRequestBlockType::SetupStage
        {
            return;
        }

        let param =
            (self.param() & !0xff) | unsafe { transmute::<SetupRequestType, u8>(new_val) } as u64;
        self.set_param(param);
    }

    pub fn setup_request_type(&self) -> Option<SetupRequestType>
    {
        if self.trb_type() != TransferRequestBlockType::SetupStage
        {
            return None;
        }

        return Some(unsafe { transmute::<u8, SetupRequestType>(self.param() as u8) });
    }

    pub fn set_setup_request(&mut self, new_val: SetupRequest)
    {
        if self.trb_type() != TransferRequestBlockType::SetupStage
        {
            return;
        }

        let param = (self.param() & !0xff00) | ((new_val as u64) << 8);
        self.set_param(param);
    }

    pub fn setup_request(&self) -> Option<SetupRequest>
    {
        if self.trb_type() != TransferRequestBlockType::SetupStage
        {
            return None;
        }

        return Some(unsafe { transmute::<u8, SetupRequest>((self.param() >> 8) as u8) });
    }

    pub fn set_setup_value(&mut self, new_val: u16)
    {
        if self.trb_type() != TransferRequestBlockType::SetupStage
        {
            return;
        }

        let param = (self.param() & !0xffff0000) | ((new_val as u64) << 16);
        self.set_param(param);
    }

    pub fn setup_value(&self) -> Option<u16>
    {
        if self.trb_type() != TransferRequestBlockType::SetupStage
        {
            return None;
        }

        return Some((self.param() >> 16) as u16);
    }

    pub fn set_setup_index(&mut self, new_val: u16)
    {
        if self.trb_type() != TransferRequestBlockType::SetupStage
        {
            return;
        }

        let param = (self.param() & !0xffff00000000) | ((new_val as u64) << 32);
        self.set_param(param);
    }

    pub fn setup_index(&self) -> Option<u16>
    {
        if self.trb_type() != TransferRequestBlockType::SetupStage
        {
            return None;
        }

        return Some((self.param() >> 32) as u16);
    }

    pub fn set_setup_length(&mut self, new_val: u16)
    {
        if self.trb_type() != TransferRequestBlockType::SetupStage
        {
            return;
        }

        let param = (self.param() & !0xffff000000000000) | ((new_val as u64) << 48);
        self.set_param(param);
    }

    pub fn setup_length(&self) -> Option<u16>
    {
        if self.trb_type() != TransferRequestBlockType::SetupStage
        {
            return None;
        }

        return Some((self.param() >> 48) as u16);
    }

    pub fn completion_code(&self) -> Option<CompletionCode>
    {
        match self.trb_type()
        {
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

        return Some(unsafe { transmute::<u8, CompletionCode>((self.status() >> 24) as u8) });
    }

    // transfer event TRB
    pub fn trb_transfer_length(&self) -> Option<usize>
    {
        if self.trb_type() != TransferRequestBlockType::TransferEvent
        {
            return None;
        }

        return Some((self.status() & 0xfff) as usize);
    }
}
