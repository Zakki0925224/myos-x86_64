use core::ptr::{read_volatile, write_volatile};

use lazy_static::lazy_static;
use modular_bitfield::{bitfield, specifiers::*, BitfieldSpecifier};
use spin::Mutex;

use crate::{arch::{addr::{PhysicalAddress, VirtualAddress}, register::cr3::Cr3}, println};

use super::bitmap::BITMAP_MEM_MAN;

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
    pub fn set_entry(
        &mut self,
        addr: &PhysicalAddress,
        is_page_table_addr: bool,
        rw: ReadWrite,
        mode: EntryMode,
        write_through_level: PageWriteThroughLevel,
    )
    {
        self.set_p(true);
        self.set_rw(rw);
        self.set_us(mode);
        self.set_pwt(write_through_level);
        self.set_disable_page_chache(false);
        self.set_accessed(true);
        self.set_is_page(!is_page_table_addr);
        self.set_restart(0);
        self.set_addr(addr.get() << 12);
        self.set_disable_execute(false);
    }

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

    pub fn map_to_identity(&self, virt_addr: &VirtualAddress)
    {
        let pml4e_index = virt_addr.get_pml4_entry_index();
        let pml3e_index = virt_addr.get_pml3_entry_index();
        let pml2e_index = virt_addr.get_pml2_entry_index();
        let pml1e_index = virt_addr.get_pml1_entry_index();

        // pml4 table
        let ptr = self.pml4_table_addr.get() as *mut PageTable;
        let mut table = unsafe { read_volatile(ptr) };
        let entry = &mut table.entries[pml4e_index];

        if !entry.is_used()
        {
            if let Some(addr) = self.calc_phys_addr(
                &BITMAP_MEM_MAN.lock().alloc_single_mem_frame().get_frame_start_virt_addr(),
            )
            {
                entry.set_entry(
                    &addr,
                    true,
                    ReadWrite::Write,
                    EntryMode::Supervisor,
                    PageWriteThroughLevel::WriteBack,
                );
            }
            else
            {
                panic!("Failed to get physical address");
            }
        }

        let entry_addr = entry.addr();
        if !entry.is_used()
        {
            unsafe { write_volatile(ptr, table) };
        }

        // pml3 table
        let ptr = (entry_addr << 12) as *mut PageTable;
        let mut table = unsafe { read_volatile(ptr) }; // TODO: PF here
        let entry = &mut table.entries[pml3e_index];

        println!("entry_addr: 0x{:x}", entry_addr);

        if !entry.is_used()
        {
            if let Some(addr) = self.calc_phys_addr(
                &BITMAP_MEM_MAN.lock().alloc_single_mem_frame().get_frame_start_virt_addr(),
            )
            {
                entry.set_entry(
                    &addr,
                    true,
                    ReadWrite::Write,
                    EntryMode::Supervisor,
                    PageWriteThroughLevel::WriteBack,
                );
            }
            else
            {
                panic!("Failed to get physical address");
            }
        }

        let entry_addr = entry.addr();
        if !entry.is_used()
        {
            unsafe { write_volatile(ptr, table) };
        }

        // pml2 table
        let ptr = (entry_addr << 12) as *mut PageTable;
        let mut table = unsafe { read_volatile(ptr) };
        let entry = &mut table.entries[pml2e_index];

        if !entry.is_used()
        {
            if let Some(addr) = self.calc_phys_addr(
                &BITMAP_MEM_MAN.lock().alloc_single_mem_frame().get_frame_start_virt_addr(),
            )
            {
                entry.set_entry(
                    &addr,
                    true,
                    ReadWrite::Write,
                    EntryMode::Supervisor,
                    PageWriteThroughLevel::WriteBack,
                );
            }
            else
            {
                panic!("Failed to get physical address");
            }
        }

        let entry_addr = entry.addr();
        if !entry.is_used()
        {
            unsafe { write_volatile(ptr, table) };
        }

        // pml1 table
        let ptr = (entry_addr << 12) as *mut PageTable;
        let mut table = unsafe { read_volatile(ptr) };
        let entry = &mut table.entries[pml1e_index];

        if !entry.is_used()
        {
            entry.set_entry(
                &PhysicalAddress::new(virt_addr.get() & !0xfff),
                false,
                ReadWrite::Write,
                EntryMode::Supervisor,
                PageWriteThroughLevel::WriteBack,
            );
            unsafe { write_volatile(ptr, table) };
        }
    }

    pub fn calc_phys_addr(&self, virt_addr: &VirtualAddress) -> Option<PhysicalAddress>
    {
        let pml4e_index = virt_addr.get_pml4_entry_index();
        let pml3e_index = virt_addr.get_pml3_entry_index();
        let pml2e_index = virt_addr.get_pml2_entry_index();
        let pml1e_index = virt_addr.get_pml1_entry_index();
        let page_offset = virt_addr.get_page_offset();

        // pml4 table
        let ptr = self.pml4_table_addr.get() as *const PageTable;
        let table = unsafe { read_volatile(ptr) };
        let entry = &table.entries[pml4e_index];

        if !entry.is_used()
        {
            return None;
        }

        // pml3 table (pdpt)
        let ptr = (entry.addr() << 12) as *const PageTable;
        let table = unsafe { read_volatile(ptr) };
        let entry = &table.entries[pml3e_index];

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

        // pml2 table (pdt)
        let ptr = (entry.addr() << 12) as *const PageTable;
        let table = unsafe { read_volatile(ptr) };
        let entry = &table.entries[pml2e_index];

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

        // pml1 table (pt)
        let ptr = (entry.addr() << 12) as *const PageTable;
        let table = unsafe { read_volatile(ptr) };
        let entry = &table.entries[pml1e_index];

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
