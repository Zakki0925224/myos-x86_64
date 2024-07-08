use crate::arch::asm;

use super::Register;

const IA32_EFER_MSR_ADDR: u32 = 0xc0000080;
const IA32_STAR_MSR_ADDR: u32 = 0xc0000081;
const IA32_LSTAR_MSR_ADDR: u32 = 0xc0000082;
const IA32_FMASK_MSR_ADDR: u32 = 0xc0000084;

#[derive(Debug, Clone, Copy)]
pub struct ExtendedFeatureEnableRegister(u64);

impl Register<u64> for ExtendedFeatureEnableRegister {
    fn read() -> Self {
        Self(asm::read_msr(IA32_EFER_MSR_ADDR))
    }

    fn write(&self) {
        asm::write_msr(IA32_EFER_MSR_ADDR, self.0)
    }

    fn raw(&self) -> u64 {
        self.0
    }

    fn set_raw(&mut self, value: u64) {
        self.0 = value;
    }
}

impl ExtendedFeatureEnableRegister {
    pub fn set_syscall_enable(&mut self, value: bool) {
        self.set_raw((self.raw() & !0x1) | (value as u64));
    }

    pub fn syscall_enable(&self) -> bool {
        (self.raw() & 0x1) != 0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LongModeSystemCallTargetAddressRegister(u64);

impl Register<u64> for LongModeSystemCallTargetAddressRegister {
    fn read() -> Self {
        Self(asm::read_msr(IA32_LSTAR_MSR_ADDR))
    }

    fn write(&self) {
        asm::write_msr(IA32_LSTAR_MSR_ADDR, self.0)
    }

    fn raw(&self) -> u64 {
        self.0
    }

    fn set_raw(&mut self, value: u64) {
        self.0 = value;
    }
}

impl LongModeSystemCallTargetAddressRegister {
    pub fn set_target_addr(&mut self, target_addr: u64) {
        self.0 = target_addr;
    }

    pub fn target_addr(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SystemCallTargetAddressRegister(u64);

impl Register<u64> for SystemCallTargetAddressRegister {
    fn read() -> Self {
        Self(asm::read_msr(IA32_STAR_MSR_ADDR))
    }

    fn write(&self) {
        asm::write_msr(IA32_STAR_MSR_ADDR, self.0)
    }

    fn raw(&self) -> u64 {
        self.0
    }

    fn set_raw(&mut self, value: u64) {
        self.0 = value;
    }
}

impl SystemCallTargetAddressRegister {
    pub fn set_target_addr(&mut self, target_addr: u64) {
        self.0 = target_addr;
    }

    pub fn target_addr(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SystemCallFlagMaskRegister(u64);

impl Register<u64> for SystemCallFlagMaskRegister {
    fn read() -> Self {
        Self(asm::read_msr(IA32_FMASK_MSR_ADDR))
    }

    fn write(&self) {
        asm::write_msr(IA32_FMASK_MSR_ADDR, self.0)
    }

    fn raw(&self) -> u64 {
        self.0
    }

    fn set_raw(&mut self, value: u64) {
        self.0 = value;
    }
}

impl SystemCallFlagMaskRegister {
    pub fn set_value(&mut self, value: u64) {
        self.0 = value;
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}
