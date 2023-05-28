use modular_bitfield::{bitfield, specifiers::*, BitfieldSpecifier};

#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[bits = 3]
pub enum EndpointState
{
    Disabled = 0,
    Running = 1,
    Halted = 2,
    Stopped = 3,
    Error = 4,
}

#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[bits = 3]
pub enum EndpointType
{
    IsochOut = 1,
    BulkOut = 2,
    InterruptOut = 3,
    ControlBidirectional = 4,
    IsochIn = 5,
    BulkIn = 6,
    InterruptIn = 7,
}

impl EndpointType
{
    pub fn new(endpoint_addr: u8, bitmap_attrs: u8) -> Self
    {
        let addr_bit7 = endpoint_addr >> 7;
        let bitmap_bit0to1 = bitmap_attrs & 0x3;

        return match (addr_bit7, bitmap_bit0to1)
        {
            (0, 1) => Self::IsochOut,
            (0, 2) => Self::BulkOut,
            (0, 3) => Self::InterruptOut,
            (_, 0) => Self::ControlBidirectional,
            (1, 1) => Self::IsochIn,
            (1, 2) => Self::BulkIn,
            _ => Self::InterruptIn,
        };
    }
}

#[bitfield]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct EndpointContext
{
    pub endpoint_state: EndpointState,
    #[skip]
    reserved0: B5,
    pub mult: B2,
    pub max_primary_streams: B5,
    pub linear_stream_array: bool,
    pub interval: B8,
    pub max_endpoint_service_time_interval_payload_high: B8,

    #[skip]
    reserved1: B1,
    pub error_cnt: B2,
    pub endpoint_type: EndpointType,
    #[skip]
    reserved2: B1,
    pub host_initiate_disable: bool,
    pub max_burst_size: B8,
    pub max_packet_size: B16,

    pub dequeue_cycle_state: bool,
    pub tr_dequeue_ptr: B63,

    pub average_trb_len: B16,
    pub max_endpoint_service_interval_payload_low: B16,

    #[skip]
    reserved3: B32,
    #[skip]
    reserved4: B32,
    #[skip]
    reserved5: B32,
}
