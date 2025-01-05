#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum EndpointType {
    IsochOut = 1,
    BulkOut = 2,
    InterruptOut = 3,
    ControlBidirectional = 4,
    IsochIn = 5,
    BulkIn = 6,
    InterruptIn = 7,
}

impl EndpointType {
    pub fn new(endpoint_addr: u8, bitmap_attrs: u8) -> Self {
        let addr_bit7 = endpoint_addr >> 7;
        let bitmap_bit0to1 = bitmap_attrs & 0x3;

        match (addr_bit7, bitmap_bit0to1) {
            (0, 1) => Self::IsochOut,
            (0, 2) => Self::BulkOut,
            (0, 3) => Self::InterruptOut,
            (_, 0) => Self::ControlBidirectional,
            (1, 1) => Self::IsochIn,
            (1, 2) => Self::BulkIn,
            _ => Self::InterruptIn,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct EndpointContext([u64; 4]);

impl EndpointContext {
    pub fn set_mult(&mut self, value: u8) {
        let value = value & 0x3; // 2 bits
        self.0[0] = (self.0[0] & !0x300) | ((value as u64) << 8);
    }

    pub fn set_max_primary_streams(&mut self, value: u8) {
        let value = value & 0x1f; // 5 bits
        self.0[0] = (self.0[0] & !0x7c00) | ((value as u64) << 10);
    }

    pub fn set_interval(&mut self, value: u8) {
        self.0[0] = (self.0[0] & !0xff_0000) | ((value as u64) << 16);
    }

    pub fn set_error_cnt(&mut self, value: u8) {
        let value = value & 0x3; // 2 bits
        self.0[0] = (self.0[0] & !0x6_0000_0000) | ((value as u64) << 33);
    }

    pub fn set_endpoint_type(&mut self, value: EndpointType) {
        self.0[0] = (self.0[0] & !0x38_0000_0000) | ((value as u64) << 35);
    }

    pub fn set_max_burst_size(&mut self, value: u8) {
        self.0[0] = (self.0[0] & !0xff00_0000_0000) | ((value as u64) << 40);
    }

    pub fn set_max_packet_size(&mut self, value: u16) {
        self.0[0] = (self.0[0] & !0xffff_0000_0000_0000) | ((value as u64) << 48);
    }

    pub fn set_dequeue_cycle_state(&mut self, value: bool) {
        self.0[1] = (self.0[1] & !0x1) | (value as u64);
    }

    pub fn set_tr_dequeue_ptr(&mut self, value: u64) {
        assert!(value & 0x1 == 0);
        self.0[1] = (self.0[1] & 0x1) | value;
    }

    pub fn set_average_trb_len(&mut self, value: u16) {
        self.0[2] = (self.0[2] & !0xffff) | (value as u64)
    }

    pub fn set_max_endpoint_service_interval_payload_low(&mut self, value: u16) {
        self.0[2] = (self.0[2] & !0xffff_0000) | ((value as u64) << 16);
    }
}
