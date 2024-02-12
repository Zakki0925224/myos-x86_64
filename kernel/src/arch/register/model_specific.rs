use crate::arch::asm;

const IA32_EFER_MSR_ADDR: u32 = 0xc0000080;
const IA32_STAR_MSR_ADDR: u32 = 0xc0000081;
const IA32_LSTAR_MSR_ADDR: u32 = 0xc0000082;
const IA32_FMASK_MSR_ADDR: u32 = 0xc0000084;

#[derive(Debug, Clone, Copy)]
pub struct ExtendedFeatureEnableRegister(u64);

impl ExtendedFeatureEnableRegister {
    pub fn read() -> Self {
        Self(asm::read_msr(IA32_EFER_MSR_ADDR))
    }

    pub fn set_system_call_enable(&mut self, value: bool) {
        self.0 = (self.0 & !0x1) | (value as u64);
    }

    pub fn write(&self) {
        asm::write_msr(IA32_EFER_MSR_ADDR, self.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LongModeSystemCallTargetAddressRegister(u64);

impl LongModeSystemCallTargetAddressRegister {
    pub fn read() -> Self {
        Self(asm::read_msr(IA32_LSTAR_MSR_ADDR))
    }

    pub fn set_target_addr(&mut self, target_addr: u64) {
        self.0 = target_addr;
    }

    pub fn write(&self) {
        asm::write_msr(IA32_LSTAR_MSR_ADDR, self.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SystemCallTargetAddressRegister(u64);

impl SystemCallTargetAddressRegister {
    pub fn read() -> Self {
        Self(asm::read_msr(IA32_STAR_MSR_ADDR))
    }

    pub fn set_target_addr(&mut self, target_addr: u64) {
        self.0 = target_addr;
    }

    pub fn write(&self) {
        asm::write_msr(IA32_STAR_MSR_ADDR, self.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SystemCallFlagMaskRegister(u64);

impl SystemCallFlagMaskRegister {
    pub fn read() -> Self {
        Self(asm::read_msr(IA32_FMASK_MSR_ADDR))
    }

    pub fn set_value(&mut self, value: u64) {
        self.0 = value;
    }

    pub fn write(&self) {
        asm::write_msr(IA32_FMASK_MSR_ADDR, self.0)
    }
}
