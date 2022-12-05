use core::ptr::read_volatile;

use lazy_static::lazy_static;
use spin::Mutex;

use crate::{arch::asm, println};

const ENTRY_LEN: usize = 512;

lazy_static! {
    pub static ref PAGING: Mutex<Paging> = Mutex::new(Paging::new());
}

#[derive(Debug)]
#[repr(C)]
pub struct PageTableEntry
{
    entry: u64,
}

impl PageTableEntry
{
    pub fn is_used(&self) -> bool { return self.entry == 0; }
}

#[derive(Debug)]
#[repr(align(4096))]
pub struct PageTable
{
    pub entries: [PageTableEntry; ENTRY_LEN],
}

#[derive(Debug)]
pub struct Paging {}

impl Paging
{
    pub fn new() -> Self { return Self {}; }

    fn read_root_page_table_start_phys_addr(&self) -> u64 { return asm::read_cr3(); }

    pub fn test(&self)
    {
        let addr = self.read_root_page_table_start_phys_addr();
        let ptr = addr as *const PageTable;
        let table = unsafe { read_volatile(ptr) };

        for (i, entry) in table.entries.iter().enumerate()
        {
            if entry.is_used()
            {
                println!("Entry {}: {:?}", i, entry);
            }
        }
    }
}
