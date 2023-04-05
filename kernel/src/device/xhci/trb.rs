use modular_bitfield::{bitfield, specifiers::*, BitfieldSpecifier};

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
pub enum TransferRequestBlockCompletionCode
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
