use modular_bitfield::{bitfield, specifiers::*, BitfieldSpecifier};

use crate::device::usb::xhc::register::PortSpeedIdValue;

#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[bits = 2]
pub enum TtThinkTime
{
    Most8FsBitTimes = 0,
    Most16FsBitTimes = 1,
    Most24FsBitTimes = 2,
    Most32FsBitTimes = 3,
}

#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[bits = 5]
pub enum SlotState
{
    DisabledOrEnabled = 0,
    Default = 1,
    Addressed = 2,
    Configured = 3,
}

#[bitfield]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SlotContext
{
    pub route_string: B20,
    pub speed: PortSpeedIdValue,
    #[skip]
    reserved0: B1,
    pub multi_tt: bool,
    pub hub: bool,
    pub context_entries: B5,

    pub max_exit_latency: B16,
    pub root_hub_port_num: B8,
    pub num_of_ports: B8,

    pub parent_hub_slot_id: B8,
    pub parent_port_num: B8,
    pub tt_think_time: TtThinkTime,
    #[skip]
    reserved1: B4,
    pub intr_target: B10,

    pub usb_device_addr: B8,
    #[skip]
    reserved2: B19,
    pub slot_state: SlotState,

    #[skip]
    reserved3: B32,
    #[skip]
    reserved4: B32,
    #[skip]
    reserved5: B32,
    #[skip]
    reserved6: B32,
}
