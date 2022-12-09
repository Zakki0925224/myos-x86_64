use core::ptr::read_volatile;

use lazy_static::lazy_static;
use modular_bitfield::{bitfield, specifiers::*, BitfieldSpecifier};
use spin::Mutex;

use crate::{arch::{addr::{PhysicalAddress, VirtualAddress}, register::cr3::Cr3}, println};

const ENTRY_LEN: usize = 512;

lazy_static! {
    pub static ref PAGING: Mutex<Paging> = Mutex::new(Paging::new());
}

#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[bits = 1]
pub enum ReadWrite
{
    Read = 0,
    Write = 1,
}

#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[bits = 1]
pub enum EntryMode
{
    Supervisor = 0,
    User = 1,
}

#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[bits = 1]
pub enum PageWriteThroughLevel
{
    WriteBack = 0,
    WriteThrough = 1,
}

#[bitfield]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct PageTableEntry
{
    p: bool,
    rw: ReadWrite,
    us: EntryMode,
    pwt: PageWriteThroughLevel,
    disable_page_chache: bool,
    accessed: bool,
    #[skip]
    ignored0: B1,
    is_page: bool,
    #[skip]
    ignored1: B3,
    restart: B1,
    addr: B51,
    disable_execute: bool,
}

impl PageTableEntry
{
    pub fn is_used(&self) -> bool { return self.p(); }
}

#[derive(Debug)]
#[repr(align(4096))]
pub struct PageTable
{
    pub entries: [PageTableEntry; ENTRY_LEN],
}

#[derive(Debug)]
pub struct Paging
{
    pml4_table_addr: PhysicalAddress,
}

impl Paging
{
    pub fn new() -> Self { return Self { pml4_table_addr: Cr3::read().pml4_table_addr }; }

    pub fn calc_phys_addr(&self, virt_addr: VirtualAddress) -> Option<PhysicalAddress>
    {
        let pml4_table_index = virt_addr.get_pml4_entry_index();
        let pml3_table_index = virt_addr.get_pml3_entry_index();
        let pml2_table_index = virt_addr.get_pml2_entry_index();
        let pml1_table_index = virt_addr.get_pml1_entry_index();
        let page_offset = virt_addr.get_page_offset();

        // pml4 table
        let ptr = self.pml4_table_addr.get() as *const PageTable;
        let table = unsafe { read_volatile(ptr) };
        let entry = &table.entries[pml4_table_index];

        if !entry.is_used()
        {
            return None;
        }

        // pdpt
        let ptr = (entry.addr() << 12) as *const PageTable;
        let table = unsafe { read_volatile(ptr) };
        let entry = &table.entries[pml3_table_index];

        if !entry.is_used()
        {
            return None;
        }

        if entry.is_page()
        {
            return Some(PhysicalAddress::new(
                ((entry.addr() & !0x3_ffff) << 12) | virt_addr.get() & 0x3fff_ffff,
            ));
        }

        // pdt
        let ptr = (entry.addr() << 12) as *const PageTable;
        let table = unsafe { read_volatile(ptr) };
        let entry = &table.entries[pml2_table_index];

        if !entry.is_used()
        {
            return None;
        }

        if entry.is_page()
        {
            return Some(PhysicalAddress::new(
                ((entry.addr() & !0x1ff) << 12) | virt_addr.get() & 0x1f_ffff,
            ));
        }

        // pt
        let ptr = (entry.addr() << 12) as *const PageTable;
        let table = unsafe { read_volatile(ptr) };
        let entry = &table.entries[pml1_table_index];

        if !entry.is_used()
        {
            return None;
        }

        if entry.is_page()
        {
            return Some(PhysicalAddress::new((entry.addr() << 12) | page_offset as u64));
        }

        return None;
    }
}
