use crate::arch::{addr::PhysicalAddress, asm};

#[derive(Debug)]
#[repr(C)]
pub struct Cr3
{
    pub pml4_table_addr: PhysicalAddress,
}

impl Cr3
{
    pub fn read() -> Self
    {
        return Self { pml4_table_addr: PhysicalAddress::new(asm::read_cr3()) };
    }

    pub fn write(&self) { asm::write_cr3(self.pml4_table_addr.get()); }
}
