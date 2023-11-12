use core::ptr::{read_volatile, write_volatile};

use crate::{error::Result, mem::paging::PAGE_MAN, util::mutex::MutexError};

use super::asm;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct PhysicalAddress(u64);

impl PhysicalAddress {
    pub const fn new(addr: u64) -> Self {
        Self(addr)
    }

    pub fn get(&self) -> u64 {
        self.0
    }

    pub fn set(&mut self, addr: u64) {
        self.0 = addr;
    }

    pub fn offset(&self, offset: usize) -> Self {
        Self::new(self.0 + offset as u64)
    }

    pub fn get_virt_addr(&self) -> VirtualAddress {
        // println!("{:?}", PAGING.lock());
        // println!("a");

        // return match PAGING.lock().mapping_type()
        // {
        //     MappingType::Identity => VirtualAddress::new(self.0),
        //     _ => panic!("Unsupported mapping type"),
        // };
        VirtualAddress::new(self.0)
    }

    pub fn out32(&self, data: u32) {
        if self.0 > u32::MAX as u64 {
            panic!("Invalid address for out32");
        }

        asm::out32(self.0 as u32, data);
    }

    pub fn in32(&self) -> u32 {
        if self.0 > u32::MAX as u64 {
            panic!("Invalid address for out32");
        }

        asm::in32(self.0 as u32)
    }
}

impl Default for PhysicalAddress {
    fn default() -> Self {
        Self(0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct VirtualAddress(u64);

impl VirtualAddress {
    pub const fn new(addr: u64) -> Self {
        Self(addr)
    }

    pub fn get(&self) -> u64 {
        self.0
    }

    pub fn set(&mut self, addr: u64) {
        self.0 = addr;
    }

    pub fn offset(&self, offset: usize) -> Self {
        Self::new(self.0 + offset as u64)
    }

    pub fn get_phys_addr(&self) -> Result<PhysicalAddress> {
        if let Some(page_man) = PAGE_MAN.try_lock() {
            return match page_man.calc_phys_addr(*self) {
                Ok(addr) => Ok(addr),
                Err(err) => Err(err),
            };
        } else {
            return Err(MutexError::Locked.into());
        }
    }

    pub fn get_pml4_entry_index(&self) -> usize {
        ((self.0 >> 39) & 0x1ff) as u16 as usize
    }

    pub fn get_pml3_entry_index(&self) -> usize {
        ((self.0 >> 30) & 0x1ff) as u16 as usize
    }

    pub fn get_pml2_entry_index(&self) -> usize {
        ((self.0 >> 21) & 0x1ff) as u16 as usize
    }

    pub fn get_pml1_entry_index(&self) -> usize {
        ((self.0 >> 12) & 0x1ff) as u16 as usize
    }

    pub fn get_page_offset(&self) -> usize {
        (self.0 & 0xfff) as u16 as usize
    }

    pub fn read_volatile<T>(&self) -> T {
        let ptr = self.get() as *const T;
        unsafe { read_volatile(ptr) }
    }

    pub fn write_volatile<T>(&self, data: T) {
        let ptr = self.get() as *mut T;
        unsafe {
            write_volatile(ptr, data);
        }
    }

    pub fn as_ptr<T>(&self) -> *const T {
        self.get() as *const T
    }

    pub fn as_ptr_mut<T>(&self) -> *mut T {
        self.get() as *mut T
    }
}

impl Default for VirtualAddress {
    fn default() -> Self {
        Self(0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct IoPortAddress(u16);

impl IoPortAddress {
    pub const fn new(addr: u16) -> Self {
        Self(addr)
    }

    pub fn get(&self) -> u16 {
        self.0
    }

    pub fn set(&mut self, addr: u16) {
        self.0 = addr;
    }

    pub fn offset(&self, offset: usize) -> Self {
        Self::new(self.0 + offset as u16)
    }

    pub fn out8(self, data: u8) {
        asm::out8(self.0 as u16, data);
    }

    pub fn in8(self) -> u8 {
        asm::in8(self.0 as u16)
    }
}

impl Default for IoPortAddress {
    fn default() -> Self {
        Self(0)
    }
}
