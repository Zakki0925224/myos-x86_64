use modular_bitfield::{bitfield, specifiers::*};

use super::device::DeviceContext;

#[bitfield]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct InputControlContext {
    drop_context_flags: B32,
    add_context_flags: B32,
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
    pub conf_value: B8,
    pub interface_num: B8,
    pub alternate_setting: B8,
    #[skip]
    reserved6: B8,
}

impl InputControlContext {
    pub fn drop_context_flag(&self, index: usize) -> Option<bool> {
        if index < 2 || index > 31 {
            return None;
        }

        return Some(((self.drop_context_flags() >> index) & 0x1) != 0);
    }

    pub fn set_drop_context_flag(&mut self, index: usize, flag: bool) -> Result<(), &'static str> {
        if index < 2 || index > 31 {
            return Err("Invalid index");
        }

        let mask = !(0x1 << index);
        let flags = (self.drop_context_flags() & mask) | (if flag { 0x1 } else { 0 } << index);
        self.set_drop_context_flags(flags);

        return Ok(());
    }

    pub fn add_context_flag(&self, index: usize) -> Option<bool> {
        if index > 31 {
            return None;
        }

        return Some(((self.add_context_flags() >> index) & 0x1) != 0);
    }

    pub fn set_add_context_flag(&mut self, index: usize, flag: bool) -> Result<(), &'static str> {
        if index > 31 {
            return Err("Invalid index");
        }

        let mask = !(0x1 << index);
        let flags = (self.add_context_flags() & mask) | (if flag { 0x1 } else { 0 } << index);
        self.set_add_context_flags(flags);

        return Ok(());
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct InputContext {
    pub input_ctrl_context: InputControlContext,
    pub device_context: DeviceContext,
}

impl InputContext {
    pub fn new() -> Self {
        return Self {
            input_ctrl_context: InputControlContext::new(),
            device_context: DeviceContext::new(),
        };
    }
}
