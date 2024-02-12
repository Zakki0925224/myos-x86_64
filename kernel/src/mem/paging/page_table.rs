const PAGE_TABLE_ENTRY_LEN: usize = 512;

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

    pub fn set_entry(
        &mut self,
        addr: u64,
        is_page_table_addr: bool,
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
        self.set_is_page(!is_page_table_addr);
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
