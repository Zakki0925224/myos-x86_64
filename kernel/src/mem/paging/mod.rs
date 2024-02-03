use self::page_table::*;
use crate::arch::addr::VirtualAddress;
use crate::arch::register::Register;
use crate::arch::{addr::*, register::control::Cr3};
use crate::error::Result;
use crate::mem::bitmap;
use crate::println;
use crate::util::mutex::{Mutex, MutexError};
use core::mem::size_of;

pub mod page_table;

pub const PAGE_SIZE: usize = 4096;
static mut PAGE_MAN: Mutex<PageManager> = Mutex::new(PageManager::new());

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappingType {
    Identity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageManagerError {
    AddressNotMappedError(VirtualAddress),
    AddressNotAlignedError(VirtualAddress),
    UnsupportedMappingTypeError(MappingType),
    InvalidPageTableEntryError(usize, PageTableEntry), // table level, entry
}

#[derive(Debug)]
pub struct PageManager {
    cr3: Cr3,
    mapping_type: MappingType,
}

impl PageManager {
    pub const fn new() -> Self {
        Self {
            cr3: Cr3::default(),
            mapping_type: MappingType::Identity,
        }
    }

    pub fn load_cr3(&mut self) {
        self.cr3 = Cr3::read();
    }

    pub fn calc_phys_addr(&self, virt_addr: VirtualAddress) -> Result<PhysicalAddress> {
        let pml4e_index = virt_addr.get_pml4_entry_index();
        let pml3e_index = virt_addr.get_pml3_entry_index();
        let pml2e_index = virt_addr.get_pml2_entry_index();
        let pml1e_index = virt_addr.get_pml1_entry_index();
        let page_offset = virt_addr.get_page_offset();

        // pml4 table
        let table: PageTable = self.pml4_table_virt_addr().read_volatile();
        let entry = table.entries[pml4e_index];

        if !entry.p() {
            return Err(PageManagerError::InvalidPageTableEntryError(4, entry).into());
        }

        // pml3 table
        let table: PageTable = VirtualAddress::new(entry.addr()).read_volatile();
        let entry = table.entries[pml3e_index];

        if !entry.p() {
            return Err(PageManagerError::InvalidPageTableEntryError(3, entry).into());
        }

        if entry.is_page() {
            return Ok(PhysicalAddress::new(
                (entry.addr() & !0x3_ffff) | virt_addr.get() & 0x3fff_ffff,
            ));
        }

        // pml2 table
        let table: PageTable = VirtualAddress::new(entry.addr()).read_volatile();
        let entry = table.entries[pml2e_index];

        if !entry.p() {
            return Err(PageManagerError::InvalidPageTableEntryError(2, entry).into());
        }

        if entry.is_page() {
            return Ok(PhysicalAddress::new(
                (entry.addr() & !0x1ff) | virt_addr.get() & 0x1f_ffff,
            ));
        }

        // pml1 table
        let table: PageTable = VirtualAddress::new(entry.addr()).read_volatile();
        let entry = table.entries[pml1e_index];

        if !entry.p() {
            return Err(PageManagerError::InvalidPageTableEntryError(1, entry).into());
        }

        if entry.is_page() {
            return Ok(PhysicalAddress::new(entry.addr() | page_offset as u64));
        }

        Err(PageManagerError::InvalidPageTableEntryError(1, entry).into())
    }

    pub fn create_new_page_table(&mut self) -> Result<()> {
        let pml4_table_virt_addr = bitmap::alloc_mem_frame(size_of::<PageTable>() / PAGE_SIZE + 1)?
            .get_frame_start_virt_addr();
        let mut pml4_page_table = pml4_table_virt_addr.read_volatile();

        let (_, total_mem_size) = bitmap::get_mem_size();
        println!("total: 0x{:x}", total_mem_size);
        println!("addr: 0x{:x}", pml4_table_virt_addr.get());

        // map_to_identityのread_volatileが良くない可能性がある
        for i in (0..=total_mem_size).step_by(0x1000) {
            self.map_to_identity(
                (i as u64).into(),
                &mut pml4_page_table,
                ReadWrite::Write,
                EntryMode::Supervisor,
                PageWriteThroughLevel::WriteBack,
            )?;
        }

        println!("ok");

        pml4_table_virt_addr.write_volatile(pml4_page_table);

        // disable current paging
        // if disable paging bit, CR3 value is fixed at 0x10033
        //let mut cr0 = Cr0::read();
        //cr0.set_paging(false);
        //cr0.write();

        // do not use PhysicalAddress
        self.cr3.set_raw(pml4_table_virt_addr.get());
        self.cr3.write();

        //cr0.set_paging(true);
        //cr0.write();

        Ok(())
    }

    pub fn set_page_permissions(
        &self,
        virt_addr: VirtualAddress,
        rw: ReadWrite,
        mode: EntryMode,
    ) -> Result<()> {
        if virt_addr.get() % PAGE_SIZE as u64 != 0 {
            return Err(PageManagerError::AddressNotAlignedError(virt_addr).into());
        }

        self.calc_phys_addr(virt_addr)?;

        let pml4e_index = virt_addr.get_pml4_entry_index();
        let pml3e_index = virt_addr.get_pml3_entry_index();
        let pml2e_index = virt_addr.get_pml2_entry_index();
        let pml1e_index = virt_addr.get_pml1_entry_index();

        let mut table_addr = self.pml4_table_virt_addr();

        // pml4 table
        let mut table: PageTable = table_addr.read_volatile();
        let entry = &mut table.entries[pml4e_index];

        if !entry.p() {
            entry.set_rw(rw);
            entry.set_us(mode);
            table_addr.write_volatile(table);
            return Ok(());
        }

        table_addr = entry.addr().into();

        // pml3 table
        let mut table: PageTable = table_addr.read_volatile();
        let entry = &mut table.entries[pml3e_index];

        if !entry.p() {
            entry.set_rw(rw);
            entry.set_us(mode);
            table_addr.write_volatile(table);
            return Ok(());
        }

        table_addr = entry.addr().into();

        // pml2
        let mut table: PageTable = table_addr.read_volatile();
        let entry = &mut table.entries[pml2e_index];

        if !entry.p() {
            entry.set_rw(rw);
            entry.set_us(mode);
            table_addr.write_volatile(table);
            return Ok(());
        }

        table_addr = entry.addr().into();

        // pml1
        let mut table: PageTable = table_addr.read_volatile();
        let entry = &mut table.entries[pml1e_index];

        // do not check present bit for rust optimization
        entry.set_rw(rw);
        entry.set_us(mode);
        table_addr.write_volatile(table);

        Ok(())
    }

    fn pml4_table_virt_addr(&self) -> VirtualAddress {
        VirtualAddress::new(self.cr3.raw())
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
            return Err(PageManagerError::UnsupportedMappingTypeError(self.mapping_type).into());
        }

        if virt_addr.get() % PAGE_SIZE as u64 != 0 {
            return Err(PageManagerError::AddressNotAlignedError(virt_addr).into());
        }

        let pml4e_index = virt_addr.get_pml4_entry_index();
        let pml3e_index = virt_addr.get_pml3_entry_index();
        let pml2e_index = virt_addr.get_pml2_entry_index();
        let pml1e_index = virt_addr.get_pml1_entry_index();

        // pml4 table
        let entry = &mut pml4_page_table.entries[pml4e_index];
        let mut entry_phys_addr = entry.addr();

        if !entry.p() {
            let mem_info = bitmap::alloc_mem_frame(size_of::<PageTable>() / PAGE_SIZE + 1)?;
            let phys_addr = mem_info.get_frame_start_virt_addr().get();
            entry.set_entry(phys_addr, true, rw, mode, write_through_level);
            entry_phys_addr = phys_addr;
        }

        // pml3 table
        let table_phys_addr = entry_phys_addr;
        let mut table: PageTable = VirtualAddress::new(table_phys_addr).read_volatile();
        let entry = &mut table.entries[pml3e_index];
        let mut entry_phys_addr = entry.addr();

        if !entry.p() {
            let mem_info = bitmap::alloc_mem_frame(size_of::<PageTable>() / PAGE_SIZE + 1)?;
            let phys_addr = mem_info.get_frame_start_virt_addr().get();

            // 1GB page
            let is_page_table_addr = !(virt_addr.get() & 0x1fff_ffff == 0);

            entry.set_entry(phys_addr, is_page_table_addr, rw, mode, write_through_level);
            entry_phys_addr = phys_addr;
            VirtualAddress::new(table_phys_addr).write_volatile(table);

            if !is_page_table_addr {
                return Ok(());
            }
        }

        // pml2 table
        let table_phys_addr = entry_phys_addr;
        let mut table: PageTable = VirtualAddress::new(table_phys_addr).read_volatile();
        let entry = &mut table.entries[pml2e_index];
        let mut entry_phys_addr = entry.addr();

        if !entry.p() {
            let mem_info = bitmap::alloc_mem_frame(size_of::<PageTable>() / PAGE_SIZE + 1)?;
            let phys_addr = mem_info.get_frame_start_virt_addr().get();

            // 2 MB page
            let is_page_table_addr = !(virt_addr.get() & 0xf_ffff == 0);

            entry.set_entry(phys_addr, is_page_table_addr, rw, mode, write_through_level);
            entry_phys_addr = phys_addr;
            VirtualAddress::new(table_phys_addr).write_volatile(table);

            if !is_page_table_addr {
                return Ok(());
            }
        }

        // pml1 table
        let table_phys_addr = entry_phys_addr;
        let mut table: PageTable = VirtualAddress::new(table_phys_addr).read_volatile();
        let entry = &mut table.entries[pml1e_index];

        if !entry.p() {
            entry.set_entry(virt_addr.get(), false, rw, mode, write_through_level);
            VirtualAddress::new(table_phys_addr).write_volatile(table);
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

pub fn set_page_permissions(
    virt_addr: VirtualAddress,
    rw: ReadWrite,
    mode: EntryMode,
) -> Result<()> {
    if let Ok(page_man) = unsafe { PAGE_MAN.try_lock() } {
        return page_man.set_page_permissions(virt_addr, rw, mode);
    }

    Err(MutexError::Locked.into())
}
