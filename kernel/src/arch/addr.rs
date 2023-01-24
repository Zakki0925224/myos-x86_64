use crate::{mem::paging::{MappingType, PAGING}, println};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct PhysicalAddress(u64);

impl PhysicalAddress
{
    pub fn new(addr: u64) -> Self { return Self { 0: addr }; }

    pub fn get(&self) -> u64 { return self.0; }

    pub fn set(&mut self, addr: u64) { self.0 = addr; }

    pub fn offset(&self, offset: usize) -> PhysicalAddress
    {
        return PhysicalAddress::new(self.0 + offset as u64);
    }

    pub fn get_virt_addr(&self) -> VirtualAddress
    {
        // println!("{:?}", PAGING.lock());
        // println!("a");

        // return match PAGING.lock().mapping_type()
        // {
        //     MappingType::Identity => VirtualAddress::new(self.0),
        //     _ => panic!("Unsupported mapping type"),
        // };
        return VirtualAddress::new(self.0);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct VirtualAddress(u64);

impl VirtualAddress
{
    pub fn is_valid_addr(addr: u64) -> bool
    {
        return !(addr > 0x7fff_ffff_ffff_ffff && addr < 0xffff_8000_0000_0000);
    }

    pub fn new(addr: u64) -> Self
    {
        if !VirtualAddress::is_valid_addr(addr)
        {
            panic!("Invalid virtual address");
        }

        return Self { 0: addr };
    }

    pub fn get(&self) -> u64 { return self.0; }

    pub fn set(&mut self, addr: u64) { self.0 = addr; }

    pub fn offset(&self, offset: usize) -> VirtualAddress
    {
        return VirtualAddress::new(self.0 + offset as u64);
    }

    pub fn get_phys_addr(&self) -> PhysicalAddress
    {
        if let Some(addr) = PAGING.lock().calc_phys_addr(self, true)
        {
            return addr;
        }

        panic!("This virtual address is not mapped (#GP)");
    }

    pub fn get_pml4_entry_index(&self) -> usize { return ((self.0 >> 39) & 0x1ff) as usize; }

    pub fn get_pml3_entry_index(&self) -> usize { return ((self.0 >> 30) & 0x1ff) as usize; }

    pub fn get_pml2_entry_index(&self) -> usize { return ((self.0 >> 21) & 0x1ff) as usize; }

    pub fn get_pml1_entry_index(&self) -> usize { return ((self.0 >> 12) & 0x1ff) as usize; }

    pub fn get_page_offset(&self) -> usize { return (self.0 & 0xfff) as usize; }
}
