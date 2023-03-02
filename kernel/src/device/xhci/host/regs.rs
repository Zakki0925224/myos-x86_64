use modular_bitfield::{bitfield, specifiers::*};

#[bitfield]
#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[repr(C)]
pub struct StructualParamaters1
{
    max_device_slots: B8,
    max_ints: B11,
    #[skip]
    reserved: B5,
    max_slots: B8,
}

#[bitfield]
#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[repr(C)]
pub struct StructualParamaters2
{
    isochronous_scheduling_threshold: B4,
    event_ring_seg_table_max: B4,
    #[skip]
    reserved: B13,
    max_scratchpad_bufs_high: B5,
    scratchpad_restore: bool,
    max_scratchpad_bufs_low: B5,
}

#[bitfield]
#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[repr(C)]
pub struct StructualParamaters3
{
    u1_device_exit_latency: B8,
    u2_device_exit_latency: B8,
    #[skip]
    reserved: B16,
}

#[bitfield]
#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[repr(C)]
pub struct CapabilityParamaters1
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
pub struct CapabilityParamaters2
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
    pub structual_params1: StructualParamaters1,
    #[skip(setters)]
    pub structual_params2: StructualParamaters2,
    #[skip(setters)]
    pub structual_params3: StructualParamaters3,
    #[skip(setters)]
    pub cap_params1: CapabilityParamaters1,
    #[skip(setters)]
    pub doorbell_offset: B32,
    #[skip(setters)]
    pub runtime_reg_space_offset: B32,
    #[skip(setters)]
    pub cap_params2: CapabilityParamaters2,
}

#[bitfield]
#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[repr(C)]
pub struct UsbCommandRegister
{
    pub run_stop: B1,
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
    ring_cycle_state: B1,
    cmd_stop: bool,
    cmd_abort: bool,
    cmd_ring_running: bool,
    #[skip]
    reserved: B2,
    cmd_ring_ptr: B58,
}

#[bitfield]
#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[repr(C)]
pub struct ConfigureRegister
{
    max_device_slots_ennabled: B8,
    u3_entry_enable: bool,
    configure_info_enable: bool,
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

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct InterruptRegisterSets
{
    pub registers: [u32; 1024],
}
