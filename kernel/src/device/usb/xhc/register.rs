use core::mem::transmute;

use modular_bitfield::{bitfield, specifiers::*, BitfieldSpecifier};

use crate::arch::addr::*;

pub const DOORBELL_REG_MAX_LEN: usize = 256;
pub const INTR_REG_SET_MAX_LEN: usize = 1024;

#[bitfield]
#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[repr(C)]
pub struct StructuralParameters1 {
    #[skip(setters)]
    pub num_of_device_slots: B8,
    #[skip(setters)]
    pub num_of_intrs: B11,
    #[skip]
    reserved: B5,
    #[skip(setters)]
    pub num_of_ports: B8,
}

#[bitfield]
#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[repr(C)]
pub struct StructuralParameters2 {
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
pub struct StructuralParameters3 {
    #[skip(setters)]
    pub u1_device_exit_latency: B8,
    #[skip(setters)]
    pub u2_device_exit_latency: B8,
    #[skip]
    reserved: B16,
}

#[bitfield]
#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[repr(C)]
pub struct CapabilityParameters1 {
    #[skip(setters)]
    pub addressing_cap_64bit: B1,
    #[skip(setters)]
    pub bandwith_negothiation_cap: bool,
    #[skip(setters)]
    pub context_size: B1,
    #[skip(setters)]
    pub port_power_control: bool,
    #[skip(setters)]
    pub port_indicators: B1,
    #[skip(setters)]
    pub light_host_controller_reset_cap: bool,
    #[skip(setters)]
    pub latency_tolerance_messaging_cap: bool,
    #[skip(setters)]
    pub no_secondary_stream_id_support: bool,
    #[skip(setters)]
    pub parse_all_event_data: B1,
    #[skip(setters)]
    pub stopped_short_packet_cap: B1,
    #[skip(setters)]
    pub stopped_edtla_cap: B1,
    #[skip(setters)]
    pub contiguous_frame_id_cap: B1,
    #[skip(setters)]
    pub max_primary_stream_array_size: B4,
    #[skip(setters)]
    pub xhci_extended_caps_pointer: B16,
}

#[bitfield]
#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[repr(C)]
pub struct CapabilityParameters2 {
    #[skip(setters)]
    pub u3_entry_cap: B1,
    #[skip(setters)]
    pub configure_endpoint_cmd_max_exit_latencty_too_large_cap: bool,
    #[skip(setters)]
    pub force_save_context_cap: B1,
    #[skip(setters)]
    pub compliance_transition_cap: B1,
    #[skip(setters)]
    pub large_esit_payload_cap: bool,
    #[skip(setters)]
    pub config_info_cap: bool,
    #[skip(setters)]
    pub extended_tbc_cap: bool,
    #[skip(setters)]
    pub extended_tbc_trb_status_cap: B1,
    #[skip(setters)]
    pub extended_property_cap: bool,
    #[skip(setters)]
    pub virtualization_base_trusted_io_cap: bool,
    #[skip]
    reserved: B22,
}

#[bitfield]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct CapabilityRegisters {
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

impl CapabilityRegisters {
    pub fn read(base_addr: VirtualAddress) -> Self {
        let mut data = [0; 8];
        for (i, field) in data.iter_mut().enumerate() {
            *field = base_addr.offset(i * 4).read_volatile::<u32>();
        }

        return unsafe { transmute::<[u32; 8], Self>(data) };
    }
}

#[bitfield]
#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[repr(C)]
pub struct UsbCommandRegister {
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
pub struct UsbStatusRegister {
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
pub struct DeviceNotificationControlRegister {
    pub notification0_enable: bool,
    pub notification1_enable: bool,
    pub notification2_enable: bool,
    pub notification3_enable: bool,
    pub notification4_enable: bool,
    pub notification5_enable: bool,
    pub notification6_enable: bool,
    pub notification7_enable: bool,
    pub notification8_enable: bool,
    pub notification9_enable: bool,
    pub notification10_enable: bool,
    pub notification11_enable: bool,
    pub notification12_enable: bool,
    pub notification13_enable: bool,
    pub notification14_enable: bool,
    pub notification15_enable: bool,
    #[skip]
    reserved: B16,
}

#[bitfield]
#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[repr(C)]
pub struct CommandRingControlRegister {
    pub ring_cycle_state: bool,
    pub cmd_stop: bool,
    pub cmd_abort: bool,
    #[skip(setters)]
    pub cmd_ring_running: bool,
    #[skip]
    reserved: B2,
    pub cmd_ring_ptr: B58,
}

#[bitfield]
#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[repr(C)]
pub struct ConfigureRegister {
    pub max_device_slots_enabled: B8,
    pub u3_entry_enable: bool,
    pub configure_info_enable: bool,
    #[skip]
    reserved: B22,
}

#[bitfield]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct OperationalRegisters {
    pub usb_cmd: UsbCommandRegister,
    pub usb_status: UsbStatusRegister,
    #[skip(setters)]
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

impl OperationalRegisters {
    pub fn read(base_addr: VirtualAddress) -> Self {
        let mut data = [0; 15];
        for (i, field) in data.iter_mut().enumerate() {
            *field = base_addr.offset(i * 4).read_volatile::<u32>();
        }

        return unsafe { transmute::<[u32; 15], Self>(data) };
    }

    pub fn write(&mut self, base_addr: VirtualAddress) {
        let mut usb_status = self.usb_status();
        usb_status.set_host_system_err(!usb_status.host_system_err());
        usb_status.set_event_int(!usb_status.event_int());
        usb_status.set_port_change_detect(!usb_status.port_change_detect());
        usb_status.set_save_restore_err(!usb_status.save_restore_err());
        self.set_usb_status(usb_status);

        let data = unsafe { transmute::<Self, [u32; 15]>(*self) };
        for (i, field) in data.iter().enumerate() {
            base_addr.offset(i * 4).write_volatile(*field);
        }
    }
}

#[bitfield]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct RuntimeRegitsers {
    #[skip(setters)]
    pub microframe_index: B14,
    #[skip]
    reserved0: B18,
    #[skip]
    reserved1: B128,
    #[skip]
    reserved1: B96,
}

impl RuntimeRegitsers {
    pub fn read(base_addr: VirtualAddress) -> Self {
        let mut data: [u32; 8] = [0; 8];
        for (i, elem) in data.iter_mut().enumerate() {
            *elem = base_addr.offset(i * 4).read_volatile::<u32>();
        }

        return unsafe { transmute::<[u32; 8], Self>(data) };
    }

    pub fn write(&self, base_addr: VirtualAddress) {
        let data = unsafe { transmute::<Self, [u32; 8]>(*self) };
        for (i, field) in data.iter().enumerate() {
            base_addr.offset(i * 4).write_volatile(*field);
        }
    }
}

#[bitfield]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct InterrupterRegisterSet {
    // interrupter management
    pub int_pending: bool,
    pub int_enable: bool,
    #[skip]
    reserved0: B30,
    // interrupter moderation
    pub int_mod_interval: B16,
    pub int_mod_counter: B16,

    // event ring registers
    pub event_ring_seg_table_size: B16,
    #[skip]
    reserved2: B16,

    #[skip]
    reserved1: B32,

    #[skip]
    reserved3: B6,
    pub event_ring_seg_table_base_addr: B58, // 64byte align
    pub dequeue_erst_seg_index: B3,
    pub event_handler_busy: bool,
    pub event_ring_dequeue_ptr: B60,
}

impl InterrupterRegisterSet {
    pub fn read(base_addr: VirtualAddress) -> Self {
        let mut data: [u32; 8] = [0; 8];
        for (i, field) in data.iter_mut().enumerate() {
            *field = base_addr.offset(i * 4).read_volatile::<u32>();
        }

        return unsafe { transmute::<[u32; 8], Self>(data) };
    }

    pub fn write(&mut self, base_addr: VirtualAddress, update_seg_table: bool) {
        self.set_int_pending(!self.int_pending());
        self.set_event_handler_busy(!self.event_handler_busy());

        let data = unsafe { transmute::<Self, [u32; 8]>(*self) };

        for (i, field) in data.iter().enumerate() {
            if i == 5 || i == 7 {
                continue;
            }

            if i == 4 || i == 6 {
                if i == 4 && !update_seg_table {
                    continue;
                }

                let qword_field = data[i] as u64 | ((data[i + 1] as u64) << 32);
                base_addr.offset(i * 4).write_volatile(qword_field);

                continue;
            }

            base_addr.offset(i * 4).write_volatile(*field);
        }
    }
}

#[bitfield]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct EventRingSegmentTableEntry {
    pub ring_seg_base_addr: B64, // 64byte alignment
    pub ring_seg_size: B16,
    #[skip]
    reserved1: B48,
}

impl EventRingSegmentTableEntry {
    pub fn is_empty(&self) -> bool {
        return self.ring_seg_base_addr() == 0 && self.ring_seg_size() == 0;
    }
}

#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[bits = 2]
pub enum PortIndicatorControl {
    Off = 0,
    Amber = 1,
    Green = 2,
    Undefined = 3,
}

#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[bits = 4]
pub enum PortSpeedIdValue {
    FullSpeed = 1,
    LowSpeed = 2,
    HighSpeed = 3,
    SuperSpeed = 4,
}

impl PortSpeedIdValue {
    pub fn get_max_packet_size(&self) -> u16 {
        return match self {
            Self::FullSpeed => 8, // or 16, 32, 64
            Self::LowSpeed => 8,
            Self::HighSpeed => 64,
            Self::SuperSpeed => 512,
        };
    }
}

#[bitfield]
#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[repr(C)]
pub struct PortStatusAndControlRegister {
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
    pub port_speed: PortSpeedIdValue,
    pub port_indicator_ctrl: PortIndicatorControl,
    pub port_link_state_write_strobe: bool,
    pub connect_status_change: bool,
    pub port_enabled_disabled_change: bool,
    pub warm_port_reset_change: bool,
    pub over_current_change: bool,
    pub port_reset_change: bool,
    pub port_link_status_change: bool,
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
pub struct PortRegisterSet {
    pub port_status_and_ctrl: PortStatusAndControlRegister,
    pub port_pm_status_and_ctrl: B32,
    #[skip(setters)]
    pub port_link_info: B32,
    pub port_hardware_lpm_ctrl: B32,
}

impl PortRegisterSet {
    pub fn read(base_addr: VirtualAddress) -> Self {
        let mut data: [u32; 4] = [0; 4];
        for (i, elem) in data.iter_mut().enumerate() {
            *elem = base_addr.offset(i * 4).read_volatile::<u32>();
        }

        return unsafe { transmute::<[u32; 4], Self>(data) };
    }

    pub fn write(&mut self, base_addr: VirtualAddress) {
        let mut port_status_and_ctrl = self.port_status_and_ctrl();
        port_status_and_ctrl
            .set_connect_status_change(!port_status_and_ctrl.connect_status_change());
        port_status_and_ctrl.set_port_reset_change(!port_status_and_ctrl.port_reset_change());
        port_status_and_ctrl
            .set_port_link_status_change(!port_status_and_ctrl.port_link_status_change());
        port_status_and_ctrl
            .set_port_config_err_change(!port_status_and_ctrl.port_config_err_change());
        self.set_port_status_and_ctrl(port_status_and_ctrl);

        let data = unsafe { transmute::<Self, [u32; 4]>(*self) };
        for (i, field) in data.iter().enumerate() {
            base_addr.offset(i * 4).write_volatile(*field);
        }
    }
}

#[bitfield]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct DoorbellRegister {
    pub db_target: B8,
    #[skip]
    reserved: B8,
    pub db_stream_id: B16,
}

impl DoorbellRegister {
    pub fn read(base_addr: VirtualAddress) -> Self {
        let data = base_addr.read_volatile::<u32>();
        return unsafe { transmute::<u32, Self>(data) };
    }

    pub fn write(&self, base_addr: VirtualAddress) {
        let data = unsafe { transmute::<Self, u32>(*self) };
        base_addr.write_volatile(data);
    }
}
