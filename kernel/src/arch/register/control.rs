use modular_bitfield::{bitfield, specifiers::*, BitfieldSpecifier};

use crate::arch::{
    addr::{PhysicalAddress, VirtualAddress},
    asm,
};

// https://en.wikipedia.org/wiki/Control_register
#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[bits = 1]
pub enum ExtensionType {
    I287 = 0,
    I387 = 1,
}

#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct Cr0 {
    pub protected_mode_enable: bool,
    pub monitor_coprocessor: bool,
    pub emulation: bool,
    pub task_switched: bool,
    pub extension_type: ExtensionType,
    pub numeric_error: bool,
    #[skip]
    reserved0: B11,
    pub write_protect: bool,
    #[skip]
    reserved1: B1,
    pub alignment_mask: bool,
    #[skip]
    reserved2: B11,
    pub not_write_through: bool,
    pub cache_disable: bool,
    pub paging: bool,
    #[skip]
    reserved3: B30,
}

impl Cr0 {
    pub fn read() -> Cr0 {
        return Cr0::from_bytes(asm::read_cr0().to_le_bytes());
    }

    pub fn write(&self) {
        asm::write_cr0(u64::from_le_bytes(self.bytes));
    }
}

pub struct Cr2;

impl Cr2 {
    pub fn read() -> VirtualAddress {
        return VirtualAddress::new(asm::read_cr2());
    }
}

pub struct Cr3;

impl Cr3 {
    pub fn read() -> PhysicalAddress {
        return PhysicalAddress::new(asm::read_cr3());
    }

    pub fn write(pml4_table_addr: PhysicalAddress) {
        asm::write_cr3(pml4_table_addr.get());
    }
}
