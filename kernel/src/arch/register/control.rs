use super::Register;
use core::arch::asm;

// https://en.wikipedia.org/wiki/Control_register
#[derive(Debug, Clone, Copy)]
pub struct Cr0(u64);
impl Register<u64> for Cr0 {
    fn read() -> Self {
        let cr0;

        unsafe {
            asm!("mov {}, cr0", out(reg) cr0);
        }

        Self(cr0)
    }

    fn write(&self) {
        unsafe {
            asm!("mov cr3, {}", in(reg) self.0);
        }
    }

    fn raw(&self) -> u64 {
        self.0
    }

    fn set_raw(&mut self, value: u64) {
        self.0 = value;
    }
}

impl Cr0 {
    pub fn set_paging(&mut self, value: bool) {
        self.0 = (self.0 & !0x8000_0000) | ((value as u64) << 31);
    }
}

pub struct Cr2(u64);

impl Register<u64> for Cr2 {
    fn read() -> Self {
        let cr2;

        unsafe {
            asm!("mov {}, cr2", out(reg) cr2);
        }

        Self(cr2)
    }

    fn write(&self) {
        panic!();
    }

    fn raw(&self) -> u64 {
        self.0
    }

    fn set_raw(&mut self, _value: u64) {
        panic!()
    }
}

#[derive(Debug)]
pub struct Cr3(u64);

impl Register<u64> for Cr3 {
    fn read() -> Self {
        let cr3;
        unsafe {
            asm!("mov {}, cr3", out(reg) cr3);
        }

        Self(cr3)
    }

    fn write(&self) {
        unsafe {
            asm!("mov cr3, {}", in(reg) self.0);
        }
    }

    fn raw(&self) -> u64 {
        self.0
    }

    fn set_raw(&mut self, value: u64) {
        self.0 = value;
    }
}

impl Cr3 {
    pub const fn new() -> Self {
        Self(0)
    }
}
