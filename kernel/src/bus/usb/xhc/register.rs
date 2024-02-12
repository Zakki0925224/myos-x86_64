use crate::arch::addr::*;
use core::mem::transmute;

pub const DOORBELL_REG_MAX_LEN: usize = 256;
pub const INTR_REG_SET_MAX_LEN: usize = 1024;

#[derive(Debug, Clone, Copy)]
#[repr(packed)]
pub struct StructuralParameters1 {
    pub num_of_device_slots: u8,
    num_of_intrs: u16,
    pub num_of_ports: u8,
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct StructuralParameters2(u32);

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct StructuralParameters3 {
    pub u1_device_exit_latency: u8,
    pub u2_device_exit_latency: u8,
    reserved: u16,
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct CapabilityParameters1(u32);

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct CapabilityParameters2(u32);

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct CapabilityRegisters {
    pub cap_reg_length: u8,
    reserved: u8,
    pub interface_version_num: u16,
    pub structural_params1: StructuralParameters1,
    pub structural_params2: StructuralParameters2,
    pub structural_params3: StructuralParameters3,
    pub cap_params1: CapabilityParameters1,
    pub doorbell_offset: u32,
    pub runtime_reg_space_offset: u32,
    pub cap_params2: CapabilityParameters2,
}

impl CapabilityRegisters {
    pub fn read(base_addr: VirtualAddress) -> Self {
        let mut data = [0; 8];
        for (i, field) in data.iter_mut().enumerate() {
            *field = base_addr.offset(i * 4).read_volatile();
        }

        unsafe { transmute::<[u32; 8], Self>(data) }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct UsbCommandRegister(u32);

impl UsbCommandRegister {
    pub fn set_run_stop(&mut self, value: bool) {
        self.0 = (self.0 & !0x1) | (value as u32);
    }

    pub fn host_controller_reset(&self) -> bool {
        (self.0 & 0x2) != 0
    }

    pub fn set_host_controller_reset(&mut self, value: bool) {
        self.0 = (self.0 & !0x2) | ((value as u32) << 1);
    }

    pub fn set_intr_enable(&mut self, value: bool) {
        self.0 = (self.0 & !0x4) | ((value as u32) << 2);
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct UsbStatusRegister(u32);

impl UsbStatusRegister {
    pub fn hchalted(&self) -> bool {
        (self.0 & 0x1) != 0
    }

    pub fn host_system_err(&self) -> bool {
        (self.0 & 0x4) != 0
    }

    pub fn set_host_system_err(&mut self, value: bool) {
        self.0 = (self.0 & !0x4) | ((value as u32) << 2);
    }

    pub fn event_int(&self) -> bool {
        (self.0 & 0x8) != 0
    }

    pub fn set_event_int(&mut self, value: bool) {
        self.0 = (self.0 & !0x8) | ((value as u32) << 3);
    }

    pub fn port_change_detect(&self) -> bool {
        (self.0 & 0x10) != 0
    }

    pub fn set_port_change_detect(&mut self, value: bool) {
        self.0 = (self.0 & !0x10) | ((value as u32) << 4);
    }

    pub fn save_restore_err(&self) -> bool {
        (self.0 & 0x400) != 0
    }

    pub fn set_save_restore_err(&mut self, value: bool) {
        self.0 = (self.0 & !0x400) | ((value as u32) << 10);
    }

    pub fn controller_not_ready(&self) -> bool {
        (self.0 & 0x800) != 0
    }

    pub fn host_controller_err(&self) -> bool {
        (self.0 & 0x1000) != 0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DeviceNotificationControlRegister(u32);

#[derive(Debug, Clone, Copy, Default)]
#[repr(transparent)]
pub struct CommandRingControlRegister(u64);

impl CommandRingControlRegister {
    pub fn set_ring_cycle_state(&mut self, value: bool) {
        self.0 = (self.0 & !0x1) | (value as u64);
    }

    pub fn set_cmd_stop(&mut self, value: bool) {
        self.0 = (self.0 & !0x2) | ((value as u64) << 1);
    }

    pub fn set_cmd_abort(&mut self, value: bool) {
        self.0 = (self.0 & !0x4) | ((value as u64) << 2);
    }

    pub fn set_cmd_ring_ptr(&mut self, value: u64) {
        assert!((value << 58) == 0);
        self.0 = (self.0 & 0xffff_ffff_ffff_ffc0) | value;
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct ConfigureRegister(u32);

impl ConfigureRegister {
    pub fn set_max_device_slots_enabled(&mut self, value: u8) {
        self.0 = (self.0 & 0xff) | (value as u32);
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct OperationalRegisters {
    pub usb_cmd: UsbCommandRegister,
    pub usb_status: UsbStatusRegister,
    pub page_size: u32,
    reserved0: [u32; 2],
    pub device_notification_ctrl: DeviceNotificationControlRegister,
    pub cmd_ring_ctrl: CommandRingControlRegister,
    reserved1: [u64; 2],
    pub device_context_base_addr_array_ptr: u64,
    pub configure: ConfigureRegister,
}

impl OperationalRegisters {
    pub fn read(base_addr: VirtualAddress) -> Self {
        let mut data = [0; 16];
        for (i, field) in data.iter_mut().enumerate() {
            *field = base_addr.offset(i * 4).read_volatile();
        }

        unsafe { transmute::<[u32; 16], Self>(data) }
    }

    pub fn write(&mut self, base_addr: VirtualAddress) {
        self.usb_status
            .set_host_system_err(!self.usb_status.host_system_err());
        self.usb_status.set_event_int(!self.usb_status.event_int());
        self.usb_status
            .set_port_change_detect(!self.usb_status.port_change_detect());
        self.usb_status
            .set_save_restore_err(!self.usb_status.save_restore_err());

        let data = unsafe { transmute::<Self, [u32; 16]>(*self) };
        for (i, field) in data.iter().enumerate() {
            base_addr.offset(i * 4).write_volatile(*field);
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct RuntimeRegitsers([u32; 8]);

impl RuntimeRegitsers {
    pub fn read(base_addr: VirtualAddress) -> Self {
        let mut data: [u32; 8] = [0; 8];
        for (i, elem) in data.iter_mut().enumerate() {
            *elem = base_addr.offset(i * 4).read_volatile();
        }

        unsafe { transmute::<[u32; 8], Self>(data) }
    }

    pub fn write(&self, base_addr: VirtualAddress) {
        for (i, field) in self.0.iter().enumerate() {
            base_addr.offset(i * 4).write_volatile(*field);
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct InterrupterRegisterSet([u64; 4]);

impl InterrupterRegisterSet {
    pub fn int_pending(&self) -> bool {
        (self.0[0] & 0x1) != 0
    }

    pub fn set_int_pending(&mut self, value: bool) {
        self.0[0] = (self.0[0] & !0x1) | (value as u64);
    }

    pub fn set_int_enable(&mut self, value: bool) {
        self.0[0] = (self.0[0] & !0x2) | ((value as u64) << 1);
    }

    pub fn set_int_mod_interval(&mut self, value: u16) {
        self.0[0] = (self.0[0] & !0xffff_0000_0000) | ((value as u64) << 32);
    }

    pub fn event_ring_seg_table_size(&self) -> u16 {
        self.0[1] as u16
    }

    pub fn set_event_ring_seg_table_size(&mut self, value: u16) {
        self.0[1] = (self.0[1] & !0xffff) | (value as u64);
    }

    pub fn event_ring_seg_table_base_addr(&self) -> u64 {
        self.0[2] & 0xffff_ffff_ffff_ffc0
    }

    pub fn set_event_ring_seg_table_base_addr(&mut self, value: u64) {
        assert!(value & 0x3f == 0);
        self.0[2] = (self.0[2] & !0xffff_ffff_ffff_ffc0) | value
    }

    pub fn dequeue_erst_seg_index(&self) -> u8 {
        self.0[3] as u8 & 0x7
    }

    pub fn set_dequeue_erst_seg_index(&mut self, value: u8) {
        let value = value & 0x7; // 3 bits
        self.0[3] = (self.0[3] & !0x7) | value as u64;
    }

    pub fn event_handler_busy(&self) -> bool {
        (self.0[3] & 0x8) != 0
    }

    pub fn set_event_handler_busy(&mut self, value: bool) {
        self.0[3] = (self.0[3] & !0x8) | ((value as u64) << 3)
    }

    pub fn event_ring_dequeue_ptr(&self) -> u64 {
        self.0[3] & 0xffff_ffff_ffff_fff0
    }

    pub fn set_event_ring_dequeue_ptr(&mut self, value: u64) {
        assert!(value & 0xf == 0);
        self.0[3] = (self.0[3] & !0xffff_ffff_ffff_fff0) | value
    }
}

impl InterrupterRegisterSet {
    pub fn read(base_addr: VirtualAddress) -> Self {
        let mut data: [u32; 8] = [0; 8];
        for (i, field) in data.iter_mut().enumerate() {
            *field = base_addr.offset(i * 4).read_volatile();
        }

        unsafe { transmute::<[u32; 8], Self>(data) }
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

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct EventRingSegmentTableEntry {
    pub ring_seg_base_addr: u64, // 64byte alignment
    pub ring_seg_size: u16,
    reserved1: [u8; 6],
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub enum PortSpeedIdValue {
    FullSpeed = 1,
    LowSpeed = 2,
    HighSpeed = 3,
    SuperSpeed = 4,
}

impl From<u8> for PortSpeedIdValue {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::FullSpeed,
            2 => Self::LowSpeed,
            3 => Self::HighSpeed,
            4 => Self::SuperSpeed,
            _ => panic!(),
        }
    }
}

impl PortSpeedIdValue {
    pub fn get_max_packet_size(&self) -> u16 {
        match self {
            Self::FullSpeed => 8, // or 16, 32, 64
            Self::LowSpeed => 8,
            Self::HighSpeed => 64,
            Self::SuperSpeed => 512,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct PortStatusAndControlRegister(u32);

impl PortStatusAndControlRegister {
    pub fn current_connect_status(&self) -> bool {
        (self.0 & 0x1) != 0
    }

    pub fn connect_status_change(&self) -> bool {
        (self.0 & 0x2_0000) != 0
    }

    pub fn set_connect_status_change(&mut self, value: bool) {
        self.0 = (self.0 & !0x2_0000) | ((value as u32) << 17);
    }

    pub fn port_reset_change(&self) -> bool {
        (self.0 & 0x20_0000) != 0
    }

    pub fn set_port_reset_change(&mut self, value: bool) {
        self.0 = (self.0 & !0x20_0000) | ((value as u32) << 19);
    }

    pub fn port_link_status_change(&self) -> bool {
        (self.0 & 0x40_0000) != 0
    }

    pub fn set_port_link_status_change(&mut self, value: bool) {
        self.0 = (self.0 & !0x40_0000) | ((value as u32) << 22);
    }

    pub fn port_config_err_change(&self) -> bool {
        (self.0 & 0x80_0000) != 0
    }

    pub fn set_port_config_err_change(&mut self, value: bool) {
        self.0 = (self.0 & !0x80_0000) | ((value as u32) << 23);
    }

    pub fn port_reset(&self) -> bool {
        (self.0 & 0x10) != 0
    }

    pub fn set_port_reset(&mut self, value: bool) {
        self.0 = (self.0 & !0x10) | ((value as u32) << 4);
    }

    pub fn port_speed(&self) -> PortSpeedIdValue {
        PortSpeedIdValue::from(((self.0 >> 10) as u8) & 0xf)
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct PortRegisterSet {
    pub port_status_and_ctrl: PortStatusAndControlRegister,
    pub port_pm_status_and_ctrl: u32,
    pub port_link_info: u32,
    pub port_hardware_lpm_ctrl: u32,
}

impl PortRegisterSet {
    pub fn read(base_addr: VirtualAddress) -> Self {
        let mut data: [u32; 4] = [0; 4];
        for (i, elem) in data.iter_mut().enumerate() {
            *elem = base_addr.offset(i * 4).read_volatile();
        }

        unsafe { transmute::<[u32; 4], Self>(data) }
    }

    pub fn write(&mut self, base_addr: VirtualAddress) {
        let mut port_status_and_ctrl = self.port_status_and_ctrl;
        port_status_and_ctrl
            .set_connect_status_change(!port_status_and_ctrl.connect_status_change());
        port_status_and_ctrl.set_port_reset_change(!port_status_and_ctrl.port_reset_change());
        port_status_and_ctrl
            .set_port_link_status_change(!port_status_and_ctrl.port_link_status_change());
        port_status_and_ctrl
            .set_port_config_err_change(!port_status_and_ctrl.port_config_err_change());
        self.port_status_and_ctrl = port_status_and_ctrl;

        let data = unsafe { transmute::<Self, [u32; 4]>(*self) };
        for (i, field) in data.iter().enumerate() {
            base_addr.offset(i * 4).write_volatile(*field);
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct DoorbellRegister {
    pub db_target: u8,
    reserved: u8,
    pub db_stream_id: u16,
}

impl DoorbellRegister {
    pub fn write(&self, base_addr: VirtualAddress) {
        base_addr.write_volatile(*self);
    }
}
