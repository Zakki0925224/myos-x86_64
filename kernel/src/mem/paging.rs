use crate::{
    arch::{
        addr::*,
        asm,
        register::{control::*, Register},
    },
    error::Result,
    mem::bitmap,
    println,
    util::mutex::*,
};

const PAGE_TABLE_ENTRY_LEN: usize = 512;
pub const PAGE_SIZE: usize = 4096;
static mut PAGE_MAN: Mutex<PageManager> = Mutex::new(PageManager::new());

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ReadWrite {
    Read = 0,
    Write = 1,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum EntryMode {
    Supervisor = 0,
    User = 1,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum PageWriteThroughLevel {
    WriteBack = 0,
    WriteThrough = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn set_p(&mut self, value: bool) {
        self.0 = (self.0 & !0x1) | (value as u64);
    }

    pub fn p(&self) -> bool {
        (self.0 & 0x1) != 0
    }

    pub fn set_rw(&mut self, rw: ReadWrite) {
        let rw = rw as u64;
        self.0 = (self.0 & !0x2) | (rw << 1);
    }

    pub fn rw(&self) -> ReadWrite {
        match (self.0 & 0x2) != 0 {
            true => ReadWrite::Write,
            false => ReadWrite::Read,
        }
    }

    pub fn set_us(&mut self, us: EntryMode) {
        let us = us as u64;
        self.0 = (self.0 & !0x4) | (us << 2);
    }

    pub fn us(&self) -> EntryMode {
        match (self.0 & 0x4) != 0 {
            true => EntryMode::User,
            false => EntryMode::Supervisor,
        }
    }

    pub fn set_pwt(&mut self, pwt: PageWriteThroughLevel) {
        let pwt = pwt as u64;
        self.0 = (self.0 & !0x8) | (pwt << 3);
    }

    pub fn set_disable_page_cache(&mut self, value: bool) {
        self.0 = (self.0 & !0x10) | ((value as u64) << 4);
    }

    pub fn set_accessed(&mut self, value: bool) {
        self.0 = (self.0 & !0x20) | ((value as u64) << 5);
    }

    pub fn set_is_page(&mut self, value: bool) {
        self.0 = (self.0 & !0x80) | ((value as u64) << 7);
    }

    pub fn is_page(&self) -> bool {
        (self.0 & 0x80) != 0
    }

    pub fn set_restart(&mut self, value: bool) {
        self.0 = (self.0 & !0x800) | ((value as u64) << 11);
    }

    pub fn set_addr(&mut self, addr: u64) {
        let addr = addr & 0x7_ffff_ffff_ffff;
        self.0 = (self.0 & !0x7fff_ffff_ffff_f000) | addr;
    }

    pub fn addr(&self) -> u64 {
        self.0 & 0x7fff_ffff_ffff_f000
    }

    pub fn set_disable_execute(&mut self, value: bool) {
        self.0 = (self.0 & !0x8000_0000_0000_0000) | ((value as u64) << 63);
    }

    pub unsafe fn page_table(&self) -> Option<&mut PageTable> {
        match self.is_page() {
            true => None,
            false => {
                let ptr = self.addr() as *mut PageTable;
                Some(&mut *ptr)
            }
        }
    }

    pub fn set_entry(
        &mut self,
        addr: u64,
        is_page: bool,
        rw: ReadWrite,
        mode: EntryMode,
        write_through_level: PageWriteThroughLevel,
    ) {
        self.set_p(true);
        self.set_rw(rw);
        self.set_us(mode);
        self.set_pwt(write_through_level);
        self.set_disable_page_cache(false);
        self.set_accessed(true);
        self.set_is_page(is_page);
        self.set_restart(false);
        self.set_addr(addr);
        self.set_disable_execute(false);
    }
}

#[derive(Debug)]
#[repr(C, align(4096))]
pub struct PageTable {
    pub entries: [PageTableEntry; PAGE_TABLE_ENTRY_LEN],
}

impl PageTable {
    pub fn new() -> Self {
        Self {
            entries: [PageTableEntry::new(); PAGE_TABLE_ENTRY_LEN],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappingType {
    Identity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageManagerError {
    AddressNotMappedError(VirtualAddress),
    AddressNotAllowedToMapError(VirtualAddress),
    AddressNotAlignedByPageSizeError(VirtualAddress),
    UnsupportedMappingTypeError(MappingType),
}

#[derive(Debug)]
pub struct PageManager {
    mapping_type: MappingType,
}

impl PageManager {
    pub const fn new() -> Self {
        Self {
            mapping_type: MappingType::Identity,
        }
    }

    pub unsafe fn calc_phys_addr(&self, virt_addr: VirtualAddress) -> Result<PhysicalAddress> {
        let pml4e_index = virt_addr.get_pml4_entry_index();
        let pml3e_index = virt_addr.get_pml3_entry_index();
        let pml2e_index = virt_addr.get_pml2_entry_index();
        let pml1e_index = virt_addr.get_pml1_entry_index();
        let page_offset = virt_addr.get_page_offset();

        let pml4_table = self.pml4_table();
        let entry = &pml4_table.entries[pml4e_index];

        if !entry.p() {
            return Err(PageManagerError::AddressNotMappedError(virt_addr).into());
        }

        let pml3_table = entry.page_table().unwrap();
        let entry = &pml3_table.entries[pml3e_index];

        if !entry.p() {
            return Err(PageManagerError::AddressNotMappedError(virt_addr).into());
        }

        let pml2_table = match entry.page_table() {
            Some(table) => table,
            // is_page == true
            None => {
                return Ok(PhysicalAddress::new(
                    (entry.addr() & !0x3_ffff) | virt_addr.get() & 0x3fff_ffff,
                ))
            }
        };
        let entry = &pml2_table.entries[pml2e_index];

        if !entry.p() {
            return Err(PageManagerError::AddressNotMappedError(virt_addr).into());
        }

        let pml1_table = match entry.page_table() {
            Some(table) => table,
            // is_page == true
            None => {
                return Ok(PhysicalAddress::new(
                    (entry.addr() & !0x1ff) | virt_addr.get() & 0x1f_ffff,
                ))
            }
        };
        let entry = &pml1_table.entries[pml1e_index];

        if !entry.is_page() {
            return Err(PageManagerError::AddressNotMappedError(virt_addr).into());
        }

        Ok(PhysicalAddress::new(entry.addr() | page_offset as u64))
    }

    pub unsafe fn create_new_page_table(
        &self,
        start: VirtualAddress,
        end: VirtualAddress,
        rw: ReadWrite,
        mode: EntryMode,
        write_through_level: PageWriteThroughLevel,
    ) -> Result<()> {
        let pml4_table_mem_frame_info = bitmap::alloc_mem_frame(1)?;
        bitmap::mem_clear(&pml4_table_mem_frame_info)?;
        let pml4_virt_addr = pml4_table_mem_frame_info.frame_start_virt_addr;
        let pml4_page_table = &mut *pml4_virt_addr.as_ptr_mut::<PageTable>();

        let (_, total_mem_size) = bitmap::get_mem_size()?;

        for i in (start.get() as usize..=total_mem_size.min(end.get() as usize)).step_by(PAGE_SIZE)
        {
            self.map_to_identity(
                (i as u64).into(),
                pml4_page_table,
                rw,
                mode,
                write_through_level,
            )?;
        }

        let mut cr3 = self.cr3();
        cr3.set_raw(pml4_virt_addr.get());
        cr3.write();

        Ok(())
    }

    pub unsafe fn get_page_permissions(
        &self,
        virt_addr: VirtualAddress,
    ) -> Result<(ReadWrite, EntryMode)> {
        if virt_addr.get() % PAGE_SIZE as u64 != 0 {
            return Err(PageManagerError::AddressNotAlignedByPageSizeError(virt_addr).into());
        }

        self.calc_phys_addr(virt_addr)?;

        let pml4e_index = virt_addr.get_pml4_entry_index();
        let pml3e_index = virt_addr.get_pml3_entry_index();
        let pml2e_index = virt_addr.get_pml2_entry_index();
        let pml1e_index = virt_addr.get_pml1_entry_index();

        let pml4_table = self.pml4_table();
        let entry = &pml4_table.entries[pml4e_index];

        if !entry.p() {
            return Err(PageManagerError::AddressNotMappedError(virt_addr).into());
        }

        let pml3_table = entry.page_table().unwrap();
        let entry = &pml3_table.entries[pml3e_index];

        if !entry.p() {
            return Err(PageManagerError::AddressNotMappedError(virt_addr).into());
        }

        let pml2_table = match entry.page_table() {
            Some(table) => table,
            None => return Ok((entry.rw(), entry.us())),
        };
        let entry = &pml2_table.entries[pml2e_index];

        if !entry.p() {
            return Err(PageManagerError::AddressNotMappedError(virt_addr).into());
        }

        let pml1_table = match entry.page_table() {
            Some(table) => table,
            None => return Ok((entry.rw(), entry.us())),
        };
        let entry = &pml1_table.entries[pml1e_index];

        if !entry.is_page() {
            return Err(PageManagerError::AddressNotMappedError(virt_addr).into());
        }

        Ok((entry.rw(), entry.us()))
    }

    pub unsafe fn set_page_permissions(
        &self,
        virt_addr: VirtualAddress,
        rw: ReadWrite,
        mode: EntryMode,
    ) -> Result<()> {
        if virt_addr.get() % PAGE_SIZE as u64 != 0 {
            return Err(PageManagerError::AddressNotAlignedByPageSizeError(virt_addr).into());
        }

        self.calc_phys_addr(virt_addr)?;

        let pml4e_index = virt_addr.get_pml4_entry_index();
        let pml3e_index = virt_addr.get_pml3_entry_index();
        let pml2e_index = virt_addr.get_pml2_entry_index();
        let pml1e_index = virt_addr.get_pml1_entry_index();

        let pml4_table = self.pml4_table();
        let entry = &pml4_table.entries[pml4e_index];

        if !entry.p() {
            return Err(PageManagerError::AddressNotMappedError(virt_addr).into());
        }

        let pml3_table = entry.page_table().unwrap();
        let entry = &mut pml3_table.entries[pml3e_index];

        if !entry.p() {
            return Err(PageManagerError::AddressNotMappedError(virt_addr).into());
        }

        let pml2_table = match entry.page_table() {
            Some(table) => table,
            None => {
                entry.set_rw(rw);
                entry.set_us(mode);
                return Ok(());
            }
        };
        let entry = &mut pml2_table.entries[pml2e_index];

        if !entry.p() {
            return Err(PageManagerError::AddressNotMappedError(virt_addr).into());
        }

        let pml1_table = match entry.page_table() {
            Some(table) => table,
            None => {
                entry.set_rw(rw);
                entry.set_us(mode);
                return Ok(());
            }
        };
        let entry = &mut pml1_table.entries[pml1e_index];

        if !entry.is_page() {
            return Err(PageManagerError::AddressNotMappedError(virt_addr).into());
        }

        entry.set_rw(rw);
        entry.set_us(mode);
        Ok(())
    }

    fn cr3(&self) -> Cr3 {
        Cr3::read()
    }

    unsafe fn pml4_table(&self) -> &mut PageTable {
        let ptr = self.cr3().raw() as *mut PageTable;
        &mut *ptr
    }

    unsafe fn map_to_identity(
        &self,
        virt_addr: VirtualAddress,
        pml4_table: &mut PageTable,
        rw: ReadWrite,
        mode: EntryMode,
        write_through_level: PageWriteThroughLevel,
    ) -> Result<()> {
        if virt_addr.get() == 0 {
            return Err(PageManagerError::AddressNotAllowedToMapError(virt_addr).into());
        }

        if self.mapping_type != MappingType::Identity {
            return Err(PageManagerError::UnsupportedMappingTypeError(self.mapping_type).into());
        }

        if virt_addr.get() % PAGE_SIZE as u64 != 0 {
            return Err(PageManagerError::AddressNotAlignedByPageSizeError(virt_addr).into());
        }

        let pml4e_index = virt_addr.get_pml4_entry_index();
        let pml3e_index = virt_addr.get_pml3_entry_index();
        let pml2e_index = virt_addr.get_pml2_entry_index();
        let pml1e_index = virt_addr.get_pml1_entry_index();

        let entry = &mut pml4_table.entries[pml4e_index];

        if !entry.p() {
            let mem_info = bitmap::alloc_mem_frame(1)?;
            bitmap::mem_clear(&mem_info)?;
            let phys_addr = mem_info.frame_start_virt_addr.get();
            entry.set_entry(phys_addr, false, rw, mode, write_through_level);
        }

        let pml3_table = entry.page_table().unwrap();
        let entry = &mut pml3_table.entries[pml3e_index];

        if !entry.p() {
            let mem_info = bitmap::alloc_mem_frame(1)?;
            bitmap::mem_clear(&mem_info)?;
            let phys_addr = mem_info.frame_start_virt_addr.get();
            entry.set_entry(phys_addr, false, rw, mode, write_through_level);
        }

        let pml2_table = entry.page_table().unwrap();
        let entry = &mut pml2_table.entries[pml2e_index];

        if !entry.p() {
            let mem_info = bitmap::alloc_mem_frame(1)?;
            bitmap::mem_clear(&mem_info)?;
            let phys_addr = mem_info.frame_start_virt_addr.get();
            entry.set_entry(phys_addr, false, rw, mode, write_through_level);
        }

        let pml1_table = entry.page_table().unwrap();
        let entry = &mut pml1_table.entries[pml1e_index];

        entry.set_entry(virt_addr.get(), true, rw, mode, write_through_level);

        Ok(())
    }
}

pub fn calc_phys_addr(virt_addr: VirtualAddress) -> Result<PhysicalAddress> {
    unsafe {
        if let Ok(page_man) = PAGE_MAN.try_lock() {
            return page_man.calc_phys_addr(virt_addr);
        }
    }

    Err(MutexError::Locked.into())
}

pub fn create_new_page_table(
    start: VirtualAddress,
    end: VirtualAddress,
    rw: ReadWrite,
    mode: EntryMode,
    write_through_level: PageWriteThroughLevel,
) -> Result<()> {
    unsafe {
        if let Ok(page_man) = PAGE_MAN.try_lock() {
            return page_man.create_new_page_table(start, end, rw, mode, write_through_level);
        }
    }

    Err(MutexError::Locked.into())
}

pub fn set_page_permissions(
    virt_addr: VirtualAddress,
    rw: ReadWrite,
    mode: EntryMode,
) -> Result<()> {
    unsafe {
        if let Ok(page_man) = PAGE_MAN.try_lock() {
            return page_man.set_page_permissions(virt_addr, rw, mode);
        }
    }

    Err(MutexError::Locked.into())
}

pub fn get_page_permissions(virt_addr: VirtualAddress) -> Result<(ReadWrite, EntryMode)> {
    unsafe {
        if let Ok(page_man) = PAGE_MAN.try_lock() {
            return page_man.get_page_permissions(virt_addr);
        }
    }

    Err(MutexError::Locked.into())
}
