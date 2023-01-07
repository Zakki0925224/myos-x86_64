use crate::arch::{addr::VirtualAddress, asm};

pub struct Cr2;

impl Cr2
{
    pub fn read() -> VirtualAddress { return VirtualAddress::new(asm::read_cr2()); }
}
