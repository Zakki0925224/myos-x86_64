use modular_bitfield::{bitfield, specifiers::*, BitfieldSpecifier};

pub const DOORBELL_REG_MAX_LEN: usize = 256;

#[bitfield]
#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[repr(C)]
pub struct StructuralParameters1
{
    #[skip(setters)]
    pub max_slots: B8,
    #[skip(setters)]
    pub max_intrs: B11,
    #[skip]
    reserved: B5,
    #[skip(setters)]
    pub max_ports: B8,
}

#[bitfield]
#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[repr(C)]
pub struct StructuralParameters2
{
    #[skip(setters)]
    pub isochronous_scheduling_threshold: B4,
    #[skip(setters)]
    pub event_ring_seg_table_max: B4,
    #[skip]
    reserved: B13,
    #[skip(setters)]
    pub max_scratchpad_bufs_high: B5,
    #[skip(setters)]
    pub scratchpad_restore: bool,
    #[skip(setters)]
    pub max_scratchpad_bufs_low: B5,
}

#[bitfield]
#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[repr(C)]
pub struct StructuralParameters3
{
    u1_device_exit_latency: B8,
    u2_device_exit_latency: B8,
    #[skip]
    reserved: B16,
}

#[bitfield]
#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[repr(C)]
pub struct CapabilityParameters1
{
    addressing_cap_64bit: B1,
    bandwith_negothiation_cap: bool,
    context_size: B1,
    port_power_control: bool,
    port_indicators: B1,
    light_host_controller_reset_cap: bool,
    latency_tolerance_messaging_cap: bool,
    no_secondary_stream_id_support: bool,
    parse_all_event_data: B1,
    stopped_short_packet_cap: B1,
    stopped_edtla_cap: B1,
    contiguous_frame_id_cap: B1,
    max_primary_stream_array_size: B4,
    xhci_extended_caps_pointer: B16,
}

#[bitfield]
#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[repr(C)]
pub struct CapabilityParameters2
{
    u3_entry_cap: B1,
    configure_endpoint_cmd_max_exit_latencty_too_large_cap: bool,
    force_save_context_cap: B1,
    compliance_transition_cap: B1,
    large_esit_payload_cap: bool,
    config_info_cap: bool,
    extended_tbc_cap: bool,
    extended_tbc_trb_status_cap: B1,
    extended_property_cap: bool,
    virtualization_base_trusted_io_cap: bool,
    #[skip]
    reserved: B22,
}

#[bitfield]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct CapabilityRegisters
{
    #[skip(setters)]
    pub cap_reg_length: B8,
    #[skip]
    reserved: B8,
    #[skip(setters)]
    pub interface_version_num: B16,
    #[skip(setters)]
    pub structural_params1: StructuralParameters1,
    #[skip(setters)]
    pub structural_params2: StructuralParameters2,
    #[skip(setters)]
    pub structural_params3: StructuralParameters3,
    #[skip(setters)]
    pub cap_params1: CapabilityParameters1,
    #[skip(setters)]
    pub doorbell_offset: B32,
    #[skip(setters)]
    pub runtime_reg_space_offset: B32,
    #[skip(setters)]
    pub cap_params2: CapabilityParameters2,
}

#[bitfield]
#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[repr(C)]
pub struct UsbCommandRegister
{
    pub run_stop: bool,
    pub host_controller_reset: bool,
    pub intr_enable: bool,
    #[skip]
    reserved0: B3,
    pub host_system_err_enable: bool,
    pub light_host_controller_reset: bool,
    pub controller_save_state: B1,
    pub controller_reset_state: B1,
    pub enable_wrap_event: bool,
    pub enable_u3_mfindex_stop: bool,
    #[skip]
    reserved1: B1,
    pub cem_enable: bool,
    #[skip(setters)]
    pub extended_tbc_enable: bool,
    #[skip(setters)]
    pub extended_tbc_trb_status_enable: bool,
    pub vtio_enable: bool,
    #[skip]
    reserved2: B15,
}

#[bitfield]
#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[repr(C)]
pub struct UsbStatusRegister
{
    #[skip(setters)]
    pub hchalted: bool,
    #[skip]
    reserved0: B1,
    pub host_system_err: bool,
    pub event_int: bool,
    pub port_change_detect: bool,
    #[skip]
    reserved1: B3,
    #[skip(setters)]
    pub save_state_status: B1,
    #[skip(setters)]
    pub restore_state_status: B1,
    pub save_restore_err: bool,
    #[skip(setters)]
    pub controller_not_ready: bool,
    #[skip(setters)]
    pub host_controller_err: bool,
    #[skip]
    reserved2: B19,
}

#[bitfield]
#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[repr(C)]
pub struct DeviceNotificationControlRegister
{
    notification0_enable: bool,
    notification1_enable: bool,
    notification2_enable: bool,
    notification3_enable: bool,
    notification4_enable: bool,
    notification5_enable: bool,
    notification6_enable: bool,
    notification7_enable: bool,
    notification8_enable: bool,
    notification9_enable: bool,
    notification10_enable: bool,
    notification11_enable: bool,
    notification12_enable: bool,
    notification13_enable: bool,
    notification14_enable: bool,
    notification15_enable: bool,
    #[skip]
    reserved: B16,
}

#[bitfield]
#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[repr(C)]
pub struct CommandRingControlRegister
{
    pub ring_cycle_state: bool,
    pub cmd_stop: bool,
    pub cmd_abort: bool,
    pub cmd_ring_running: bool,
    #[skip]
    reserved: B2,
    pub cmd_ring_ptr: B58,
}

#[bitfield]
#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[repr(C)]
pub struct ConfigureRegister
{
    pub max_device_slots_enabled: B8,
    pub u3_entry_enable: bool,
    pub configure_info_enable: bool,
    #[skip]
    reserved: B22,
}

#[bitfield]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct OperationalRegisters
{
    pub usb_cmd: UsbCommandRegister,
    pub usb_status: UsbStatusRegister,
    pub page_size: B32,
    #[skip]
    reserved0: B64,
    pub device_notification_ctrl: DeviceNotificationControlRegister,
    pub cmd_ring_ctrl: CommandRingControlRegister,
    #[skip]
    reserved1: B128,
    pub device_context_base_addr_array_ptr: B64,
    pub configure: ConfigureRegister,
}

#[bitfield]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct RuntimeRegitsers
{
    microframe_index: B32,
    #[skip]
    reserved0: B32,
    #[skip]
    reserved1: B32,
    #[skip]
    reserved2: B32,
    #[skip]
    reserved3: B32,
    #[skip]
    reserved4: B32,
    #[skip]
    reserved5: B32,
    #[skip]
    reserved6: B32,
}

#[bitfield]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct InterrupterRegisterSet
{
    // interrupter management
    pub int_pending: bool,
    pub int_enable: bool,
    #[skip]
    reserved0: B30,
    // interrupter moderation
    pub int_mod_interval: B16,
    pub int_mod_counter: B16,

    pub event_ring_seg_table_size: B16,
    #[skip]
    reserved1: B54,
    pub event_ring_seg_table_base_addr: B58,
    pub dequeue_erst_seg_index: B3,
    pub event_handler_busy: bool,
    pub event_ring_dequeue_ptr: B60,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct InterrupterRegisterSets
{
    pub registers: [InterrupterRegisterSet; 1024],
}

#[bitfield]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct EventRingSegmentTableEntry
{
    #[skip]
    reserved0: B6,
    pub ring_seg_base_addr: B58,
    pub ring_seg_size: B16,
    #[skip]
    reserved1: B48,
}

impl EventRingSegmentTableEntry
{
    pub fn is_empty(&self) -> bool
    {
        return self.ring_seg_base_addr() == 0 && self.ring_seg_size() == 0;
    }
}

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

#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[bits = 2]
pub enum PortIndicatorControl
{
    Off = 0,
    Amber = 1,
    Green = 2,
    Undefined = 3,
}

#[bitfield]
#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[repr(C)]
pub struct PortStatusAndControlRegister
{
    #[skip(setters)]
    pub current_connect_status: bool,
    pub port_enabled: bool,
    #[skip]
    reserved0: B1,
    #[skip(setters)]
    pub over_current_active: bool,
    pub port_reset: bool,
    pub port_link_state: B4,
    pub port_power: bool,
    #[skip(setters)]
    pub port_speed: B4,
    pub port_indicator_ctrl: PortIndicatorControl,
    pub port_link_state_write_strobe: bool,
    pub connect_status_change: bool,
    pub port_enabled_disabled_change: bool,
    pub warm_port_reset_change: bool,
    pub over_current_change: bool,
    pub port_reset_change: bool,
    pub port_link_state_change: bool,
    pub port_config_err_change: bool,
    #[skip(setters)]
    pub cold_attach_status: bool,
    pub wake_on_connect_enable: bool,
    pub wake_on_disconnect_enable: bool,
    pub wake_on_over_current_enable: bool,
    #[skip]
    reserved1: B2,
    #[skip(setters)]
    pub device_removable: bool,
    pub warm_port_reset: bool,
}

#[bitfield]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct PortRegisterSet
{
    pub port_status_and_ctrl: PortStatusAndControlRegister,
    pub port_pm_status_and_ctrl: B32,
    #[skip(setters)]
    pub port_link_info: B32,
    pub port_hardware_lpm_ctrl: B32,
}

#[bitfield]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct DoorbellRegister
{
    pub db_target: B8,
    #[skip]
    reserved: B8,
    pub db_stream_id: B8,
}
