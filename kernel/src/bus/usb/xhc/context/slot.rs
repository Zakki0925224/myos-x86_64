use crate::bus::usb::xhc::register::PortSpeedIdValue;

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct SlotContext([u32; 8]);

impl SlotContext {
    pub fn set_speed(&mut self, value: PortSpeedIdValue) {
        self.0[0] = (self.0[0] & !0xf0_0000) | ((value as u32) << 20);
    }

    pub fn set_context_entries(&mut self, value: u8) {
        let value = value & 0x1f; // 5 bits
        self.0[0] = (self.0[0] & !0xf800_0000) | ((value as u32) << 26);
    }

    pub fn set_root_hub_port_num(&mut self, value: u8) {
        self.0[1] = (self.0[1] & !0xff_0000) | ((value as u32) << 16);
    }
}
