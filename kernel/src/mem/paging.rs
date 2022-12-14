use core::ptr::{read_volatile, write_volatile};

use lazy_static::lazy_static;
use log::{error, info};
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
        self.set_addr(addr.get() >> 12);
        self.set_disable_execute(false);
    }

    pub fn get_addr(&self) -> PhysicalAddress { return PhysicalAddress::new(self.addr() << 12); }

    pub fn is_used(&self) -> bool { return self.p(); }
}

#[derive(Debug)]
#[repr(align(4096))]
pub struct PageTable
{
    pub entries: [PageTableEntry; ENTRY_LEN],
}

#[derive(Debug, Clone, Copy)]
pub enum MappingType
{
    Identity,
}

#[derive(Debug)]
pub struct Paging
{
    pml4_table_addr: PhysicalAddress,
    pml4_table_addr_backup: PhysicalAddress,
    mapping_type: MappingType,
}

impl Paging
{
    pub fn new() -> Self
    {
        let table_addr = Cr3::read();

        return Self {
            pml4_table_addr: table_addr,
            pml4_table_addr_backup: table_addr,
            mapping_type: MappingType::Identity,
        };
    }

    pub fn create_new_page_table(&mut self)
    {
        if let Some(mem_info) = BITMAP_MEM_MAN.lock().alloc_single_mem_frame()
        {
            self.pml4_table_addr =
                self.calc_phys_addr(&mem_info.get_frame_start_virt_addr(), true).unwrap();
        }
        else
        {
            error!("Failed to allocate memory frame for PML4 table");
            return;
        }

        let total_mem_size = BITMAP_MEM_MAN.lock().get_total_mem_size();
        let total_mem_size: usize = 0xf800000; // can go to end
        let total_mem_size: usize = 0xf800001; // stop anywhere in loop
        let mut virt_addr = VirtualAddress::new(0);

        while virt_addr.get() < total_mem_size as u64
        {
            if let Err(_) = self.map_to_identity(&virt_addr)
            {
                self.pml4_table_addr = self.pml4_table_addr_backup;
                error!("Failed to create new page table");
                return;
            }

            // check
            if self.calc_phys_addr(&virt_addr, false) == None
            {
                self.pml4_table_addr = self.pml4_table_addr_backup;
                error!("New page tables was not work collectly");
                return;
            }

            virt_addr.set(virt_addr.get() + BITMAP_MEM_MAN.lock().get_frame_size() as u64);
        }

        info!("Finished to map to identity");
        Cr3::write(self.pml4_table_addr);
        self.pml4_table_addr_backup = self.pml4_table_addr;
        self.mapping_type = MappingType::Identity;
        info!("Switched to new page table");
    }

    pub fn map_to_identity(&self, virt_addr: &VirtualAddress) -> Result<(), ()>
    {
        let pml4e_index = virt_addr.get_pml4_entry_index();
        let pml3e_index = virt_addr.get_pml3_entry_index();
        let pml2e_index = virt_addr.get_pml2_entry_index();
        let pml1e_index = virt_addr.get_pml1_entry_index();

        // pml4 table
        let ptr = self.pml4_table_addr.get_virt_addr().get() as *mut PageTable;
        let mut table = unsafe { read_volatile(ptr) };
        let entry = &mut table.entries[pml4e_index];
        let mut entry_addr = entry.get_addr().get_virt_addr();

        if !entry.is_used()
        {
            if let Some(mem_info) = BITMAP_MEM_MAN.lock().alloc_single_mem_frame()
            {
                let addr = mem_info.get_frame_start_virt_addr();
                // cannot use addr.get_phys_addr() at here
                entry.set_entry(
                    &self.calc_phys_addr(&addr, true).unwrap(),
                    true,
                    ReadWrite::Write,
                    EntryMode::Supervisor,
                    PageWriteThroughLevel::WriteBack,
                );

                //println!("new PML4 entry[{}]: {:?}", pml4e_index, entry);
                entry_addr = addr;
                unsafe { write_volatile(ptr, table) }
            }
            else
            {
                error!("Failed to allocate memory frame for PML4 entry");
                return Err(());
            }
        }

        // pml3 table
        let ptr = entry_addr.get() as *mut PageTable;
        let mut table = unsafe { read_volatile(ptr) };
        let entry = &mut table.entries[pml3e_index];
        let mut entry_addr = entry.get_addr().get_virt_addr();

        if !entry.is_used()
        {
            if let Some(mem_info) = BITMAP_MEM_MAN.lock().alloc_single_mem_frame()
            {
                let mut addr =
                    self.calc_phys_addr(&mem_info.get_frame_start_virt_addr(), true).unwrap();
                let mut is_page_table_addr = true;

                // 1GB page
                if virt_addr.get() & 0x1fff_ffff == 0
                {
                    addr = PhysicalAddress::new(addr.get());
                    is_page_table_addr = false;
                }

                entry.set_entry(
                    &addr,
                    is_page_table_addr,
                    ReadWrite::Write,
                    EntryMode::Supervisor,
                    PageWriteThroughLevel::WriteBack,
                );

                //println!("new PML3 entry[{}]: {:?}", pml4e_index, entry);
                entry_addr = mem_info.get_frame_start_virt_addr();
                unsafe { write_volatile(ptr, table) }

                if !is_page_table_addr
                {
                    return Ok(());
                }
            }
            else
            {
                error!("Failed to allocate memory frame for PML3 entry");
                return Err(());
            }
        }

        // pml2 table
        let ptr = entry_addr.get() as *mut PageTable;
        let mut table = unsafe { read_volatile(ptr) };
        let entry = &mut table.entries[pml2e_index];
        let mut entry_addr = entry.get_addr().get_virt_addr();

        if !entry.is_used()
        {
            if let Some(mem_info) = BITMAP_MEM_MAN.lock().alloc_single_mem_frame()
            {
                let mut addr =
                    self.calc_phys_addr(&mem_info.get_frame_start_virt_addr(), true).unwrap();
                let mut is_page_table_addr = true;

                // 2MB page
                if virt_addr.get() & 0xf_ffff == 0
                {
                    addr = PhysicalAddress::new(addr.get());
                    is_page_table_addr = false;
                }

                entry.set_entry(
                    &addr,
                    is_page_table_addr,
                    ReadWrite::Write,
                    EntryMode::Supervisor,
                    PageWriteThroughLevel::WriteBack,
                );

                //println!("new PML2 entry[{}]: {:?}", pml4e_index, entry);
                entry_addr = mem_info.get_frame_start_virt_addr();
                unsafe { write_volatile(ptr, table) }

                if !is_page_table_addr
                {
                    return Ok(());
                }
            }
            else
            {
                error!("Failed to allocate memory frame for PML2 entry");
                return Err(());
            }
        }

        // pml1 table
        let ptr = entry_addr.get() as *mut PageTable;
        let mut table = unsafe { read_volatile(ptr) };
        let entry = &mut table.entries[pml1e_index];

        if !entry.is_used()
        {
            entry.set_entry(
                &PhysicalAddress::new(virt_addr.get()),
                false,
                ReadWrite::Write,
                EntryMode::Supervisor,
                PageWriteThroughLevel::WriteBack,
            );

            //println!("new PML1 entry[{}]: {:?}", pml1e_index, entry);
            unsafe { write_volatile(ptr, table) }
        }

        return Ok(());
    }

    pub fn calc_phys_addr(
        &self,
        virt_addr: &VirtualAddress,
        is_use_backup_pml4_table_addr: bool,
    ) -> Option<PhysicalAddress>
    {
        let pml4e_index = virt_addr.get_pml4_entry_index();
        let pml3e_index = virt_addr.get_pml3_entry_index();
        let pml2e_index = virt_addr.get_pml2_entry_index();
        let pml1e_index = virt_addr.get_pml1_entry_index();
        let page_offset = virt_addr.get_page_offset();

        // pml4 table
        let ptr = if is_use_backup_pml4_table_addr
        {
            self.pml4_table_addr_backup
        }
        else
        {
            self.pml4_table_addr
        }
        .get_virt_addr()
        .get() as *const PageTable;
        let table = unsafe { read_volatile(ptr) };
        let entry = &table.entries[pml4e_index];

        //println!("get PML4 entry[{}]: {:?}", pml4e_index, entry);

        if !entry.is_used()
        {
            return None;
        }

        // pml3 table (pdpt)
        let ptr = entry.get_addr().get_virt_addr().get() as *const PageTable;
        let table = unsafe { read_volatile(ptr) };
        let entry = &table.entries[pml3e_index];

        //println!("get PML3 entry[{}]: {:?}", pml3e_index, entry);

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
        let ptr = entry.get_addr().get_virt_addr().get() as *const PageTable;
        let table = unsafe { read_volatile(ptr) };
        let entry = &table.entries[pml2e_index];

        //println!("get PML2 entry[{}]: {:?}", pml2e_index, entry);

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
        let ptr = entry.get_addr().get_virt_addr().get() as *const PageTable;
        let table = unsafe { read_volatile(ptr) };
        let entry = &table.entries[pml1e_index];

        //println!("get PML1 entry[{}]: {:?}", pml1e_index, entry);

        if !entry.is_used()
        {
            return None;
        }

        if entry.is_page()
        {
            return Some(PhysicalAddress::new(entry.addr() << 12 | page_offset as u64));
        }

        return None;
    }

    pub fn mapping_type(&self) -> MappingType { return self.mapping_type; }
}
