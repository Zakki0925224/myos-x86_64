use crate::arch::addr::VirtualAddress;
use crate::arch::register::control::Cr0;
use crate::arch::{addr::*, register::control::Cr3};
use crate::error::Result;
use crate::println;
use crate::util::mutex::{Mutex, MutexError};

use self::page_table::*;

use super::bitmap::BITMAP_MEM_MAN;

pub mod page_table;

static mut PAGE_MAN: Mutex<PageManager> = Mutex::new(PageManager::new());

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappingType {
    Identity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageManagerError {
    AddressNotMappedError(VirtualAddress),
    UnsupportedMappingTypeError(MappingType),
    InvalidPageTableEntryError(usize, PageTableEntry), // table level, entry
}

#[derive(Debug)]
pub struct PageManager {
    pml4_table_virt_addr: VirtualAddress,
    mapping_type: MappingType,
}

impl PageManager {
    pub const fn new() -> Self {
        Self {
            pml4_table_virt_addr: VirtualAddress::new(0),
            mapping_type: MappingType::Identity,
        }
    }

    pub fn load_cr3(&mut self) {
        self.pml4_table_virt_addr = Cr3::read().get_virt_addr();
    }

    pub fn calc_phys_addr(&self, virt_addr: VirtualAddress) -> Result<PhysicalAddress> {
        let pml4e_index = virt_addr.get_pml4_entry_index();
        let pml3e_index = virt_addr.get_pml3_entry_index();
        let pml2e_index = virt_addr.get_pml2_entry_index();
        let pml1e_index = virt_addr.get_pml1_entry_index();
        let page_offset = virt_addr.get_page_offset();

        // pml4 table
        let table: PageTable = self.pml4_table_virt_addr.read_volatile();
        let entry = table.entries[pml4e_index];

        if !entry.p() {
            return Err(PageManagerError::InvalidPageTableEntryError(4, entry).into());
        }

        // pml3 table
        let table: PageTable = entry.get_phys_addr().get_virt_addr().read_volatile();
        let entry = table.entries[pml3e_index];

        if !entry.p() {
            return Err(PageManagerError::InvalidPageTableEntryError(3, entry).into());
        }

        if entry.is_page() {
            return Ok(PhysicalAddress::new(
                ((entry.addr() & !0x3_ffff) << 12) | virt_addr.get() & 0x3fff_ffff,
            ));
        }

        // pml2 table
        let table: PageTable = entry.get_phys_addr().get_virt_addr().read_volatile();
        let entry = table.entries[pml2e_index];

        if !entry.p() {
            return Err(PageManagerError::InvalidPageTableEntryError(2, entry).into());
        }

        if entry.is_page() {
            return Ok(PhysicalAddress::new(
                ((entry.addr() & !0x1ff) << 12) | virt_addr.get() & 0x1f_ffff,
            ));
        }

        // pml1 table
        let table: PageTable = entry.get_phys_addr().get_virt_addr().read_volatile();
        let entry = table.entries[pml1e_index];

        if !entry.p() {
            return Err(PageManagerError::InvalidPageTableEntryError(1, entry).into());
        }

        if entry.is_page() {
            return Ok(PhysicalAddress::new(
                entry.addr() << 12 | page_offset as u64,
            ));
        }

        Err(PageManagerError::InvalidPageTableEntryError(1, entry).into())
    }

    pub fn create_new_page_table(&mut self) -> Result<()> {
        let pml4_table_virt_addr = BITMAP_MEM_MAN
            .try_lock()
            .unwrap()
            .alloc_single_mem_frame()?
            .get_frame_start_virt_addr();
        let mut pml4_page_table = PageTable::new();

        let total_mem_size = BITMAP_MEM_MAN.try_lock().unwrap().get_total_mem_size();
        println!("total: 0x{:x}", total_mem_size);
        //let total_mem_size = 0x0a000000 as usize;
        //let total_mem_size = 0x09000000 as usize;
        let mut virt_addr = VirtualAddress::default();

        while virt_addr.get() < total_mem_size as u64 {
            // info!(
            //     "mem: Mapping {}%...",
            //     ((virt_addr.get() as f64) / (total_mem_size as f64)) * 100f64
            // );

            if let Err(err) = self.map_to_identity(
                virt_addr,
                &mut pml4_page_table,
                ReadWrite::Write,
                EntryMode::Supervisor,
                PageWriteThroughLevel::WriteBack,
            ) {
                return Err(err);
            }

            virt_addr = virt_addr.offset(PAGE_SIZE);
        }

        pml4_table_virt_addr.write_volatile(pml4_page_table);
        let pml4_table_phys_addr = self.calc_phys_addr(pml4_table_virt_addr).unwrap();

        // disable current paging
        let mut cr0 = Cr0::read();
        cr0.set_paging(false);
        cr0.write();
        Cr3::write(pml4_table_phys_addr);
        cr0.set_paging(true);
        cr0.write();

        Ok(())
    }

    fn map_to_identity(
        &self,
        virt_addr: VirtualAddress,
        pml4_page_table: &mut PageTable,
        rw: ReadWrite,
        mode: EntryMode,
        write_through_level: PageWriteThroughLevel,
    ) -> Result<()> {
        if self.mapping_type != MappingType::Identity {
            return Err(
                PageManagerError::UnsupportedMappingTypeError(self.mapping_type.clone()).into(),
            );
        }

        let pml4e_index = virt_addr.get_pml4_entry_index();
        let pml3e_index = virt_addr.get_pml3_entry_index();
        let pml2e_index = virt_addr.get_pml2_entry_index();
        let pml1e_index = virt_addr.get_pml1_entry_index();

        // pml4 table
        let mut entry = pml4_page_table.entries[pml4e_index];
        let mut entry_phys_addr = entry.get_phys_addr();

        if !entry.p() {
            let mem_info = BITMAP_MEM_MAN
                .try_lock()
                .unwrap()
                .alloc_single_mem_frame()?;
            let phys_addr = self.calc_phys_addr(mem_info.get_frame_start_virt_addr())?;
            entry.set_entry(phys_addr, true, rw, mode, write_through_level);
            entry_phys_addr = phys_addr;
            pml4_page_table.entries[pml4e_index] = entry;
        }

        // pml3 table
        let table_phys_addr = entry_phys_addr;
        let mut table: PageTable = table_phys_addr.get_virt_addr().read_volatile();
        let mut entry = table.entries[pml3e_index];
        let mut entry_phys_addr = entry.get_phys_addr();

        if !entry.p() {
            let mem_info = BITMAP_MEM_MAN
                .try_lock()
                .unwrap()
                .alloc_single_mem_frame()?;
            let phys_addr = self.calc_phys_addr(mem_info.get_frame_start_virt_addr())?;

            // 1GB page
            let is_page_table_addr = !(virt_addr.get() & 0x1fff_ffff == 0);

            entry.set_entry(phys_addr, is_page_table_addr, rw, mode, write_through_level);
            entry_phys_addr = phys_addr;
            table.entries[pml3e_index] = entry;
            table_phys_addr.get_virt_addr().write_volatile(table);

            if !is_page_table_addr {
                return Ok(());
            }
        }

        // pml2 table
        let table_phys_addr = entry_phys_addr;
        let mut table: PageTable = table_phys_addr.get_virt_addr().read_volatile();
        let mut entry = table.entries[pml2e_index];
        let mut entry_phys_addr = entry.get_phys_addr();

        if !entry.p() {
            let mem_info = BITMAP_MEM_MAN
                .try_lock()
                .unwrap()
                .alloc_single_mem_frame()?;
            let phys_addr = self.calc_phys_addr(mem_info.get_frame_start_virt_addr())?;

            // 2 MB page
            let is_page_table_addr = !(virt_addr.get() & 0xf_ffff == 0);

            entry.set_entry(phys_addr, is_page_table_addr, rw, mode, write_through_level);
            entry_phys_addr = phys_addr;
            table.entries[pml2e_index] = entry;
            table_phys_addr.get_virt_addr().write_volatile(table);

            if !is_page_table_addr {
                return Ok(());
            }
        }

        // pml1 table
        let table_phys_addr = entry_phys_addr;
        let mut table: PageTable = table_phys_addr.get_virt_addr().read_volatile();
        let mut entry = table.entries[pml3e_index];

        if !entry.p() {
            entry.set_entry(
                PhysicalAddress::new(virt_addr.get()),
                false,
                rw,
                mode,
                write_through_level,
            );
            table.entries[pml1e_index] = entry;
            table_phys_addr.get_virt_addr().write_volatile(table);
        }

        Ok(())
    }
}

pub fn load_cr3() -> Result<()> {
    if let Ok(mut page_man) = unsafe { PAGE_MAN.try_lock() } {
        page_man.load_cr3();
        return Ok(());
    }

    Err(MutexError::Locked.into())
}

pub fn calc_phys_addr(virt_addr: VirtualAddress) -> Result<PhysicalAddress> {
    if let Ok(page_man) = unsafe { PAGE_MAN.try_lock() } {
        return page_man.calc_phys_addr(virt_addr);
    }

    Err(MutexError::Locked.into())
}

pub fn create_new_page_table() -> Result<()> {
    if let Ok(mut page_man) = unsafe { PAGE_MAN.try_lock() } {
        return page_man.create_new_page_table();
    }

    Err(MutexError::Locked.into())
}
