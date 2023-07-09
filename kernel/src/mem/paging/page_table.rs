use modular_bitfield::*;
use modular_bitfield::{specifiers::*, BitfieldSpecifier};

use crate::arch::addr::*;

const PAGE_TABLE_ENTRY_LEN: usize = 512;
pub const PAGE_SIZE: usize = 4096;

#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[bits = 1]
pub enum ReadWrite {
    Read = 0,
    Write = 1,
}

#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[bits = 1]
pub enum EntryMode {
    Supervisor = 0,
    User = 1,
}

#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[bits = 1]
pub enum PageWriteThroughLevel {
    WriteBack = 0,
    WriteThrough = 1,
}

#[bitfield]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct PageTableEntry {
    pub p: bool,
    pub rw: ReadWrite,
    pub us: EntryMode,
    pub pwt: PageWriteThroughLevel,
    pub disable_page_chache: bool,
    pub accessed: bool,
    #[skip]
    pub reserved0: B1,
    pub is_page: bool,
    #[skip]
    pub reserved1: B3,
    pub restart: B1,
    pub addr: B51,
    pub disable_execute: bool,
}

impl PageTableEntry {
    pub fn set_entry(
        &mut self,
        addr: PhysicalAddress,
        is_page_table_addr: bool,
        rw: ReadWrite,
        mode: EntryMode,
        write_through_level: PageWriteThroughLevel,
    ) {
        self.set_p(true);
        self.set_rw(rw);
        self.set_us(mode);
        self.set_pwt(write_through_level);
        self.set_disable_page_chache(false);
        self.set_accessed(true);
        self.set_is_page(!is_page_table_addr);
        self.set_restart(0);
        self.set_addr(addr.get() >> 12);
        self.set_disable_execute(false);
    }

    pub fn get_phys_addr(&self) -> PhysicalAddress {
        return PhysicalAddress::new(self.addr() << 12);
    }
}

impl Default for PageTableEntry {
    fn default() -> Self {
        Self {
            bytes: Default::default(),
        }
    }
}

#[derive(Debug)]
#[repr(C, align(4096))]
pub struct PageTable {
    pub entries: [PageTableEntry; PAGE_TABLE_ENTRY_LEN],
}

impl PageTable {
    pub fn new() -> Self {
        return Self {
            entries: [PageTableEntry::default(); PAGE_TABLE_ENTRY_LEN],
        };
    }
}
