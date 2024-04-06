use core::ptr::{read_volatile, write_volatile};

use super::asm;
use crate::{error::Result, mem::paging};

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

    pub fn get_virt_addr(&self) -> Result<VirtualAddress> {
        paging::calc_virt_addr(*self)
    }
}

impl Default for PhysicalAddress {
    fn default() -> Self {
        Self(0)
    }
}

impl From<u64> for PhysicalAddress {
    fn from(addr: u64) -> Self {
        PhysicalAddress::new(addr)
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
        paging::calc_phys_addr(*self)
    }

    pub fn get_pml4_entry_index(&self) -> usize {
        ((self.0 >> 39) & 0x1ff) as usize
    }

    pub fn get_pml3_entry_index(&self) -> usize {
        ((self.0 >> 30) & 0x1ff) as usize
    }

    pub fn get_pml2_entry_index(&self) -> usize {
        ((self.0 >> 21) & 0x1ff) as usize
    }

    pub fn get_pml1_entry_index(&self) -> usize {
        ((self.0 >> 12) & 0x1ff) as usize
    }

    pub fn get_page_offset(&self) -> usize {
        (self.0 & 0xfff) as usize
    }

    #[deprecated]
    pub fn read_volatile<T>(&self) -> T {
        let ptr = self.get() as *const T;
        unsafe { read_volatile(ptr) }
    }

    #[deprecated]
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

    pub fn copy_from_nonoverlapping<T>(&self, src: *const T, count: usize) {
        unsafe { self.as_ptr_mut::<T>().copy_from_nonoverlapping(src, count) }
    }
}

impl Default for VirtualAddress {
    fn default() -> Self {
        Self(0)
    }
}

impl From<u64> for VirtualAddress {
    fn from(addr: u64) -> Self {
        VirtualAddress::new(addr)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct IoPortAddress(u32);

impl IoPortAddress {
    pub const fn new(addr: u32) -> Self {
        Self(addr)
    }

    pub fn get(&self) -> u32 {
        self.0
    }

    pub fn set(&mut self, addr: u32) {
        self.0 = addr;
    }

    pub fn offset(&self, offset: usize) -> Self {
        Self::new(self.0 + offset as u32)
    }

    pub fn out8(&self, value: u8) {
        assert!(self.0 <= u16::MAX as u32);

        asm::out8(self.0 as u16, value);
    }

    pub fn in8(&self) -> u8 {
        assert!(self.0 <= u16::MAX as u32);

        asm::in8(self.0 as u16)
    }

    pub fn out32(&self, value: u32) {
        asm::out32(self.0, value);
    }

    pub fn in32(&self) -> u32 {
        asm::in32(self.0)
    }
}

impl Default for IoPortAddress {
    fn default() -> Self {
        Self(0)
    }
}
