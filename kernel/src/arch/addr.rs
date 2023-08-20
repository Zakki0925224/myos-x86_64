use core::ptr::{read_volatile, write_volatile};

use crate::mem::paging::PAGE_MAN;

pub trait Address {
    fn new(addr: u64) -> Self;
    fn get(&self) -> u64;
    fn set(&mut self, addr: u64);
    fn offset(&self, offset: usize) -> Self;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct PhysicalAddress(u64);

impl Address for PhysicalAddress {
    fn new(addr: u64) -> Self {
        return Self(addr);
    }

    fn get(&self) -> u64 {
        return self.0;
    }

    fn set(&mut self, addr: u64) {
        self.0 = addr;
    }

    fn offset(&self, offset: usize) -> Self {
        return Self::new(self.0 + offset as u64);
    }
}

impl PhysicalAddress {
    pub fn get_virt_addr(&self) -> VirtualAddress {
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

impl Default for PhysicalAddress {
    fn default() -> Self {
        return Self(0);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct VirtualAddress(u64);

impl Address for VirtualAddress {
    fn new(addr: u64) -> Self {
        return Self(addr);
    }

    fn get(&self) -> u64 {
        return self.0;
    }

    fn set(&mut self, addr: u64) {
        self.0 = addr;
    }

    fn offset(&self, offset: usize) -> Self {
        return Self::new(self.0 + offset as u64);
    }
}

impl VirtualAddress {
    pub fn is_valid_addr(addr: u64) -> bool {
        return !(addr > 0x7fff_ffff_ffff_ffff && addr < 0xffff_8000_0000_0000);
    }

    pub fn get_phys_addr(&self) -> PhysicalAddress {
        if PAGE_MAN.is_locked() {
            panic!("Page manager is locked");
        }

        match PAGE_MAN.lock().calc_phys_addr(*self) {
            Ok(addr) => return addr,
            Err(err) => panic!("mem: {:?}", err),
        }
    }

    pub fn get_pml4_entry_index(&self) -> usize {
        return ((self.0 >> 39) & 0x1ff) as u16 as usize;
    }

    pub fn get_pml3_entry_index(&self) -> usize {
        return ((self.0 >> 30) & 0x1ff) as u16 as usize;
    }

    pub fn get_pml2_entry_index(&self) -> usize {
        return ((self.0 >> 21) & 0x1ff) as u16 as usize;
    }

    pub fn get_pml1_entry_index(&self) -> usize {
        return ((self.0 >> 12) & 0x1ff) as u16 as usize;
    }

    pub fn get_page_offset(&self) -> usize {
        return (self.0 & 0xfff) as u16 as usize;
    }

    pub fn read_volatile<T>(&self) -> T {
        let ptr = self.get() as *const T;
        return unsafe { read_volatile(ptr) };
    }

    pub fn write_volatile<T>(&self, data: T) {
        let ptr = self.get() as *mut T;
        unsafe {
            write_volatile(ptr, data);
        }
    }
}

impl Default for VirtualAddress {
    fn default() -> Self {
        return Self(0);
    }
}
