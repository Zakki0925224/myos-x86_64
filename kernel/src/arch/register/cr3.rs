use crate::arch::{addr::PhysicalAddress, asm};

pub struct Cr3;

impl Cr3
{
    pub fn read() -> PhysicalAddress { return PhysicalAddress::new(asm::read_cr3()); }

    pub fn write(pml4_table_addr: PhysicalAddress) { asm::write_cr3(pml4_table_addr.get()); }
}
