use super::device::DeviceContext;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct InputControlContext {
    drop_context_flags: u32,
    add_context_flags: u32,
    reserved1: [u32; 5],
    pub conf_value: u8,
    pub interface_num: u8,
    pub alternate_setting: u8,
    reserved2: u8,
}

impl InputControlContext {
    pub const fn new() -> Self {
        Self {
            drop_context_flags: 0,
            add_context_flags: 0,
            reserved1: [0; 5],
            conf_value: 0,
            interface_num: 0,
            alternate_setting: 0,
            reserved2: 0,
        }
    }

    pub fn drop_context_flag(&self, index: usize) -> Option<bool> {
        if index < 2 || index > 31 {
            return None;
        }

        Some(((self.drop_context_flags >> index) & 0x1) != 0)
    }

    pub fn set_drop_context_flag(&mut self, index: usize, flag: bool) -> Result<(), &'static str> {
        if index < 2 || index > 31 {
            return Err("Invalid index");
        }

        let mask = !(0x1 << index);
        let flags = (self.drop_context_flags & mask) | (if flag { 0x1 } else { 0 } << index);
        self.drop_context_flags = flags;

        Ok(())
    }

    pub fn add_context_flag(&self, index: usize) -> Option<bool> {
        if index > 31 {
            return None;
        }

        Some(((self.add_context_flags >> index) & 0x1) != 0)
    }

    pub fn set_add_context_flag(&mut self, index: usize, flag: bool) -> Result<(), &'static str> {
        if index > 31 {
            return Err("Invalid index");
        }

        let mask = !(0x1 << index);
        let flags = (self.add_context_flags & mask) | (if flag { 0x1 } else { 0 } << index);
        self.add_context_flags = flags;

        Ok(())
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
        Self {
            input_ctrl_context: InputControlContext::new(),
            device_context: DeviceContext::new(),
        }
    }
}
