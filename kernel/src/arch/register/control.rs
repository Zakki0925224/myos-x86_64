use crate::arch::{addr::*, asm};

// https://en.wikipedia.org/wiki/Control_register
#[derive(Debug, Clone, Copy)]
pub struct Cr0(u64);

impl Cr0 {
    pub fn read() -> Self {
        Self(asm::read_cr0())
    }

    pub fn write(&self) {
        asm::write_cr0(self.0);
    }

    pub fn set_paging(&mut self, value: bool) {
        let value = if value { 0x1 } else { 0x0 };
        self.0 = (self.0 & !0x8000_0000) | (value << 31);
    }
}

pub struct Cr2;

impl Cr2 {
    pub fn read() -> VirtualAddress {
        VirtualAddress::new(asm::read_cr2())
    }
}

pub struct Cr3;

impl Cr3 {
    pub fn read() -> PhysicalAddress {
        PhysicalAddress::new(asm::read_cr3())
    }

    pub fn write(pml4_table_addr: u64) {
        asm::write_cr3(pml4_table_addr);
    }
}
