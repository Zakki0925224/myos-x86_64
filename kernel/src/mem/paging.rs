use crate::{
    arch::{
        addr::*,
        register::{control::*, Register},
    },
    error::Result,
    mem::bitmap,
    util::mutex::*,
};
use alloc::string::String;
use log::info;

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

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct PageTableEntry(u64);

impl core::fmt::Debug for PageTableEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut fmt = String::from("PageTableEntry");
        fmt = format!(
            "{}(0x{:x}) {{ p: {}, rw: {:?}, us: {:?}, a: {}, d: {}, page_size: {}, addr: 0x{:x}, xd: {} }}",
            fmt,
            self.0,
            self.p(),
            self.rw(),
            self.us(),
            self.accessed(),
            self.dirty(),
            self.page_size(),
            self.addr(),
            self.exec_disable()
        );
        write!(f, "{}", fmt)
    }
}

impl PageTableEntry {
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

    pub fn pwt(&self) -> PageWriteThroughLevel {
        match (self.0 & 0x8) != 0 {
            true => PageWriteThroughLevel::WriteThrough,
            false => PageWriteThroughLevel::WriteBack,
        }
    }

    pub fn accessed(&self) -> bool {
        (self.0 & 0x20) != 0
    }

    pub fn dirty(&self) -> bool {
        (self.0 & 0x40) != 0
    }

    // must be 0, unsupported >4KB page table
    pub fn page_size(&self) -> bool {
        let value = (self.0 & 0x80) != 0;
        assert!(!value);
        value
    }

    pub fn set_addr(&mut self, addr: u64) {
        assert!((addr & !0xf_ffff_ffff_f000) == 0);
        self.0 = self.0 | addr;
    }

    pub fn addr(&self) -> u64 {
        self.0 & 0xf_ffff_ffff_f000
    }

    pub fn exec_disable(&self) -> bool {
        (self.0 & (1 << 63)) != 0
    }

    pub unsafe fn page_table(&self) -> Option<&mut PageTable> {
        match self.page_size() {
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
        rw: ReadWrite,
        mode: EntryMode,
        write_through_level: PageWriteThroughLevel,
    ) {
        self.set_p(true);
        self.set_rw(rw);
        self.set_us(mode);
        self.set_pwt(write_through_level);
        self.set_addr(addr);
    }
}

#[derive(Debug)]
#[repr(C, align(4096))]
pub struct PageTable {
    pub entries: [PageTableEntry; PAGE_TABLE_ENTRY_LEN],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageManagerError {
    AddressNotMappedError(VirtualAddress),
    AddressNotAllowedToMapError(VirtualAddress),
    AddressNotAlignedByPageSizeError(VirtualAddress),
}

#[derive(Debug)]
pub struct PageManager;

impl PageManager {
    pub const fn new() -> Self {
        Self
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
            None => {
                unimplemented!();
            }
        };
        let entry = &pml2_table.entries[pml2e_index];

        if !entry.p() {
            return Err(PageManagerError::AddressNotMappedError(virt_addr).into());
        }

        let pml1_table = match entry.page_table() {
            Some(table) => table,
            None => {
                unimplemented!();
            }
        };
        let entry = &pml1_table.entries[pml1e_index];
        Ok(PhysicalAddress::new(entry.addr() | page_offset as u64))
    }

    pub unsafe fn create_new_page_table(
        &self,
        start: VirtualAddress,
        end: VirtualAddress,
        phys_addr: PhysicalAddress,
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
            self.set_map(
                (i as u64).into(),
                phys_addr.offset(i - phys_addr.get() as usize),
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

    pub unsafe fn update_mapping(
        &self,
        start: VirtualAddress,
        end: VirtualAddress,
        phys_addr: PhysicalAddress,
        rw: ReadWrite,
        mode: EntryMode,
        write_through_level: PageWriteThroughLevel,
    ) -> Result<()> {
        let pml4_virt_addr = VirtualAddress::new(self.cr3().raw());
        let pml4_page_table = &mut *pml4_virt_addr.as_ptr_mut::<PageTable>();

        for i in (start.get() as usize..=end.get() as usize).step_by(PAGE_SIZE) {
            self.set_map(
                (i as u64).into(),
                phys_addr.offset(i - phys_addr.get() as usize),
                pml4_page_table,
                rw,
                mode,
                write_through_level,
            )?;
        }

        Ok(())
    }

    pub unsafe fn page_table_entry(&self, virt_addr: VirtualAddress) -> Result<&PageTableEntry> {
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

        let pml2_table = entry.page_table().unwrap();
        let entry = &pml2_table.entries[pml2e_index];

        if !entry.p() {
            return Err(PageManagerError::AddressNotMappedError(virt_addr).into());
        }

        let pml1_table = entry.page_table().unwrap();
        let entry = &pml1_table.entries[pml1e_index];
        Ok(entry)
    }

    fn cr3(&self) -> Cr3 {
        Cr3::read()
    }

    unsafe fn pml4_table(&self) -> &mut PageTable {
        let ptr = self.cr3().raw() as *mut PageTable;
        &mut *ptr
    }

    unsafe fn set_map(
        &self,
        virt_addr: VirtualAddress,
        phys_addr: PhysicalAddress,
        pml4_table: &mut PageTable,
        rw: ReadWrite,
        mode: EntryMode,
        write_through_level: PageWriteThroughLevel,
    ) -> Result<()> {
        if virt_addr.get() == 0 {
            return Err(PageManagerError::AddressNotAllowedToMapError(virt_addr).into());
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
            let mut new_entry = PageTableEntry::default();
            new_entry.set_entry(
                mem_info.frame_start_virt_addr.get(),
                rw,
                mode,
                write_through_level,
            );
            *entry = new_entry;
        }

        let pml3_table = entry.page_table().unwrap();
        let entry = &mut pml3_table.entries[pml3e_index];

        if !entry.p() {
            let mem_info = bitmap::alloc_mem_frame(1)?;
            bitmap::mem_clear(&mem_info)?;
            let mut new_entry = PageTableEntry::default();
            new_entry.set_entry(
                mem_info.frame_start_virt_addr.get(),
                rw,
                mode,
                write_through_level,
            );
            *entry = new_entry;
        }

        let pml2_table = entry.page_table().unwrap();
        let entry = &mut pml2_table.entries[pml2e_index];

        if !entry.p() {
            let mem_info = bitmap::alloc_mem_frame(1)?;
            bitmap::mem_clear(&mem_info)?;
            let mut new_entry = PageTableEntry::default();
            new_entry.set_entry(
                mem_info.frame_start_virt_addr.get(),
                rw,
                mode,
                write_through_level,
            );
            *entry = new_entry;
        }

        let pml1_table = entry.page_table().unwrap();
        let entry = &mut pml1_table.entries[pml1e_index];

        entry.set_entry(phys_addr.get(), rw, mode, write_through_level);
        let mut new_entry = PageTableEntry::default();
        new_entry.set_entry(phys_addr.get(), rw, mode, write_through_level);
        *entry = new_entry;

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
    phys_addr: PhysicalAddress,
    rw: ReadWrite,
    mode: EntryMode,
    write_through_level: PageWriteThroughLevel,
) -> Result<()> {
    unsafe {
        if let Ok(page_man) = PAGE_MAN.try_lock() {
            return page_man.create_new_page_table(
                start,
                end,
                phys_addr,
                rw,
                mode,
                write_through_level,
            );
        }
    }

    Err(MutexError::Locked.into())
}

pub fn update_mapping(
    start: VirtualAddress,
    end: VirtualAddress,
    phys_addr: PhysicalAddress,
    rw: ReadWrite,
    mode: EntryMode,
    write_through_level: PageWriteThroughLevel,
) -> Result<()> {
    unsafe {
        if let Ok(page_man) = PAGE_MAN.try_lock() {
            page_man.update_mapping(start, end, phys_addr, rw, mode, write_through_level)?;
            info!(
                "paging: Updated mapping (virt 0x{:x}-0x{:x} -> phys 0x{:x}-0x{:x})",
                start.get(),
                end.get(),
                phys_addr.get(),
                phys_addr.offset((end.get() - start.get()) as usize).get()
            );

            return Ok(());
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
            let entry = page_man.page_table_entry(virt_addr)?;
            return page_man.update_mapping(
                virt_addr,
                virt_addr,
                entry.addr().into(),
                rw,
                mode,
                entry.pwt(),
            );
        }
    }

    Err(MutexError::Locked.into())
}

pub fn get_page_permissions(virt_addr: VirtualAddress) -> Result<(ReadWrite, EntryMode)> {
    unsafe {
        if let Ok(page_man) = PAGE_MAN.try_lock() {
            let entry = page_man.page_table_entry(virt_addr)?;
            return Ok((entry.rw(), entry.us()));
        }
    }

    Err(MutexError::Locked.into())
}

pub fn read_page_table_entry(virt_addr: VirtualAddress) -> Result<PageTableEntry> {
    unsafe {
        if let Ok(page_man) = PAGE_MAN.try_lock() {
            let entry = page_man.page_table_entry(virt_addr)?;
            return Ok(entry.clone());
        }
    }

    Err(MutexError::Locked.into())
}
