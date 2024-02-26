use super::paging::{
    self,
    page_table::{EntryMode, ReadWrite},
    PAGE_SIZE,
};
use crate::{
    arch::addr::*,
    error::Result,
    util::mutex::{Mutex, MutexError},
};
use alloc::vec::Vec;
use common::mem_desc::*;
use core::mem::size_of;
use log::info;

static mut BITMAP_MEM_MAN: Mutex<Option<BitmapMemoryManager>> = Mutex::new(None);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MemoryFrameInfo {
    frame_start_virt_addr: VirtualAddress,
    frame_size: usize, // must be 4096B align
    frame_index: usize,
    is_allocated: bool,
}

impl MemoryFrameInfo {
    pub fn get_frame_start_virt_addr(&self) -> VirtualAddress {
        self.frame_start_virt_addr
    }

    pub fn get_frame_start_phys_addr(&self) -> PhysicalAddress {
        self.frame_start_virt_addr.get_phys_addr().unwrap()
    }

    pub fn get_frame_size(&self) -> usize {
        self.frame_size
    }

    pub fn get_frame_index(&self) -> usize {
        self.frame_index
    }

    pub fn is_allocated(&self) -> bool {
        self.is_allocated
    }

    pub fn set_permissions_to_supervisor(&self) -> Result<()> {
        self.set_permissions(ReadWrite::Write, EntryMode::Supervisor)
    }

    pub fn set_permissions_to_user(&self) -> Result<()> {
        self.set_permissions(ReadWrite::Write, EntryMode::User)
    }

    pub fn set_permissions(&self, rw: ReadWrite, mode: EntryMode) -> Result<()> {
        let page_len = self.frame_size / PAGE_SIZE;
        let mut start_virt_addr = self.frame_start_virt_addr;

        for _ in 0..page_len {
            paging::set_page_permissions(start_virt_addr, rw, mode)?;
            start_virt_addr = start_virt_addr.offset(PAGE_SIZE);
        }

        Ok(())
    }

    pub fn get_permissions(&self) -> Result<Vec<(ReadWrite, EntryMode)>> {
        let page_len = self.frame_size / PAGE_SIZE;
        let mut start_virt_addr = self.frame_start_virt_addr;
        let mut res = Vec::new();

        for _ in 0..page_len {
            res.push(paging::get_page_permissions(start_virt_addr)?);
            start_virt_addr = start_virt_addr.offset(PAGE_SIZE);
        }

        Ok(res)
    }
}

const BITMAP_SIZE: usize = u8::BITS as usize;
#[derive(Debug)]
struct Bitmap(u8);

impl Bitmap {
    pub fn new(bitmap: u8) -> Self {
        Self(bitmap)
    }

    pub fn get_map(&self) -> [bool; BITMAP_SIZE] {
        let mut map = [false; BITMAP_SIZE];

        for i in 0..BITMAP_SIZE {
            map[i] = ((self.0 << i) & 0x80) != 0;
        }

        map
    }

    pub fn set_map(&mut self, map: [bool; BITMAP_SIZE]) {
        let mut bitmap = 0;
        for i in 0..BITMAP_SIZE {
            bitmap |= (map[i] as u8) << BITMAP_SIZE - 1 - i;
        }

        self.0 = bitmap;
    }

    pub fn is_allocated_all(&self) -> bool {
        self.0 == 0xff
    }

    pub fn is_free_all(&self) -> bool {
        self.0 == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitmapMemoryManagerError {
    NotInitialized,
    AllocateMemoryForBitmapError,
    FailedToReadBitmapError,
    FreeMemoryFrameWasNotFoundError,
    MemoryFrameIndexOutOfBoundsError(usize), // memory frame index
    MemoryMapOffsetOutOfBounds(usize),       // memory map offset
    MemoryFrameWasAlreadyAllocatedError(usize), // memory frame index,
    MemoryFrameWasAlreadyDeallocatedError(usize), // memory frame index,
    InvalidMemoryFrameLengthError(usize),    // memory frame length
}

#[derive(Debug)]
pub struct BitmapMemoryManager {
    bitmap_virt_addr: VirtualAddress,
    bitmap_len: usize,
    frame_len: usize,
    allocated_frame_len: usize,
    free_frame_len: usize,
    frame_size: usize,
}

impl BitmapMemoryManager {
    pub fn new(mem_map: &[MemoryDescriptor]) -> Result<Self> {
        // TODO: boot services data/code
        // get total page count (a page=4096B)
        let total_page_cnt = mem_map.into_iter().map(|d| d.page_cnt as usize).sum();
        // get bitmap len
        let mut bitmap_len = total_page_cnt / BITMAP_SIZE;

        if total_page_cnt % BITMAP_SIZE != 0 {
            bitmap_len += 1;
        }

        // find available memory area for bitmap
        let mut bitmap_virt_addr = VirtualAddress::default();
        for d in mem_map {
            if d.ty != MemoryType::Conventional
                || d.page_cnt as usize * UEFI_PAGE_SIZE < bitmap_len
                || d.phys_start == 0
            {
                continue;
            }

            bitmap_virt_addr.set(d.phys_start);
            break;
        }

        if bitmap_virt_addr.get() == 0 {
            return Err(BitmapMemoryManagerError::AllocateMemoryForBitmapError.into());
        }

        let mut bmm = Self {
            bitmap_virt_addr,
            bitmap_len,
            frame_len: total_page_cnt,
            allocated_frame_len: 0,
            free_frame_len: total_page_cnt,
            frame_size: UEFI_PAGE_SIZE,
        };

        // clear all bitmap
        bmm.clear_bitmap();

        // allocate no conventional memory frame
        let mut frame_index = 0;

        for d in mem_map {
            if d.ty == MemoryType::Conventional {
                frame_index += d.page_cnt as usize;
                continue;
            }

            for _ in 0..d.page_cnt {
                bmm.alloc_frame(frame_index)?;
                frame_index += 1;
            }
        }

        // allocate bitmap memory frame
        let start = bmm.get_mem_frame_index(bitmap_virt_addr);
        let end = bmm.get_mem_frame_index(bitmap_virt_addr.offset(bitmap_len));
        for i in start..=end {
            bmm.alloc_frame(i)?;
        }

        // allocate less 1MB memory space
        let start = 0;
        let end = bmm.get_mem_frame_index(VirtualAddress::new(1024 * 1024));
        for i in start..=end {
            if let Err(_) = bmm.alloc_frame(i) {
                // already allocated
                continue;
            }
        }

        info!("mem: Initialized bitmap memory manager");

        Ok(bmm)
    }

    pub fn init(&mut self, mem_map: &[MemoryDescriptor]) -> Result<()> {
        // TODO: boot services data/code
        // get total page count (a page=4096B)
        let total_page_cnt = mem_map.into_iter().map(|d| d.page_cnt as usize).sum();
        // get bitmap len
        let mut bitmap_len = total_page_cnt / BITMAP_SIZE;

        if total_page_cnt % BITMAP_SIZE != 0 {
            bitmap_len += 1;
        }

        // find available memory area for bitmap
        let mut bitmap_virt_addr = VirtualAddress::default();
        for d in mem_map {
            if d.ty != MemoryType::Conventional
                || d.page_cnt as usize * UEFI_PAGE_SIZE < bitmap_len
                || d.phys_start == 0
            {
                continue;
            }

            bitmap_virt_addr.set(d.phys_start);
            break;
        }

        if bitmap_virt_addr.get() == 0 {
            return Err(BitmapMemoryManagerError::AllocateMemoryForBitmapError.into());
        }

        self.bitmap_virt_addr = bitmap_virt_addr;
        self.bitmap_len = bitmap_len;
        self.frame_len = total_page_cnt;
        self.allocated_frame_len = 0;
        self.free_frame_len = self.frame_len;
        self.frame_size = UEFI_PAGE_SIZE;

        // clear all bitmap
        self.clear_bitmap();

        // allocate no conventional memory frame
        let mut frame_index = 0;

        for d in mem_map {
            if d.ty == MemoryType::Conventional {
                frame_index += d.page_cnt as usize;
                continue;
            }

            for _ in 0..d.page_cnt {
                self.alloc_frame(frame_index)?;
                frame_index += 1;
            }
        }

        // allocate bitmap memory frame
        let start = self.get_mem_frame_index(self.bitmap_virt_addr);
        let end = self.get_mem_frame_index(self.bitmap_virt_addr.offset(self.bitmap_len));
        for i in start..=end {
            self.alloc_frame(i)?;
        }

        // allocate less 1MB memory space
        let start = 0;
        let end = self.get_mem_frame_index(VirtualAddress::new(1024 * 1024));
        for i in start..=end {
            if let Err(_) = self.alloc_frame(i) {
                // already allocated
                continue;
            }
        }

        info!("mem: Initialized bitmap memory manager");

        Ok(())
    }

    pub fn get_frame_size(&self) -> usize {
        self.frame_size
    }

    pub fn get_total_mem_size(&self) -> usize {
        self.frame_size * self.frame_len
    }

    pub fn get_used_mem_size(&self) -> usize {
        self.allocated_frame_len * self.frame_size
    }

    pub fn get_mem_frame(&self, frame_index: usize) -> Option<MemoryFrameInfo> {
        if frame_index >= self.frame_len {
            return None;
        }

        let bitmap_offset = frame_index / BITMAP_SIZE;
        let bitmap_pos = frame_index % BITMAP_SIZE; // 0 ~ 7
        if let Some(bitmap) = self.read_bitmap(bitmap_offset) {
            return Some(MemoryFrameInfo {
                frame_start_virt_addr: VirtualAddress::new((frame_index * self.frame_size) as u64),
                frame_size: self.frame_size,
                frame_index,
                is_allocated: bitmap.get_map()[bitmap_pos],
            });
        }

        None
    }

    pub fn alloc_single_mem_frame(&mut self) -> Result<MemoryFrameInfo> {
        if self.free_frame_len == 0 {
            return Err(BitmapMemoryManagerError::FreeMemoryFrameWasNotFoundError.into());
        }

        let mut found_mem_frame_index = None;
        'outer: for i in 0..self.bitmap_len {
            if let Some(bitmap) = self.read_bitmap(i) {
                if bitmap.is_allocated_all() {
                    continue;
                }

                for j in 0..BITMAP_SIZE {
                    if !bitmap.get_map()[j] {
                        found_mem_frame_index = Some(i * BITMAP_SIZE + j);
                        break 'outer;
                    }
                }
            } else {
                return Err(BitmapMemoryManagerError::FailedToReadBitmapError.into());
            }
        }

        if let None = found_mem_frame_index {
            return Err(BitmapMemoryManagerError::FreeMemoryFrameWasNotFoundError.into());
        }

        let found_mem_frame_index = found_mem_frame_index.unwrap();

        let mem_frame_info = MemoryFrameInfo {
            frame_start_virt_addr: VirtualAddress::new(
                (found_mem_frame_index * self.frame_size) as u64,
            ),
            frame_size: self.frame_size,
            frame_index: found_mem_frame_index,
            is_allocated: true,
        };

        self.alloc_frame(found_mem_frame_index)?;
        mem_frame_info.set_permissions(ReadWrite::Write, EntryMode::Supervisor)?;
        self.mem_clear(&mem_frame_info);

        Ok(mem_frame_info)
    }

    pub fn alloc_multi_mem_frame(&mut self, len: usize) -> Result<MemoryFrameInfo> {
        if len == 0 {
            return Err(BitmapMemoryManagerError::InvalidMemoryFrameLengthError(len).into());
        }

        if len == 1 {
            return self.alloc_single_mem_frame();
        }

        let mut start = None;
        let mut count = 0;
        let mut i = 0;

        'outer: while i < self.bitmap_len {
            if let Some(bitmap) = self.read_bitmap(i) {
                let map = bitmap.get_map();
                for j in 0..BITMAP_SIZE {
                    if start != None && count == len {
                        break 'outer;
                    }

                    if len - count > BITMAP_SIZE && bitmap.is_free_all() && start != None {
                        count += BITMAP_SIZE;
                        break;
                    }

                    if len - count > BITMAP_SIZE && bitmap.is_allocated_all() {
                        start = None;
                        count = 0;
                        break;
                    }

                    // free
                    if !map[j] {
                        count += 1;

                        if start.is_none() {
                            start = Some((i, j));
                        }
                    }
                    // already allocated
                    else {
                        start = None;
                        count = 0;
                    }
                }

                i += 1;
            } else {
                return Err(BitmapMemoryManagerError::FailedToReadBitmapError.into());
            }
        }

        if start.is_none() || count != len {
            return Err(BitmapMemoryManagerError::FreeMemoryFrameWasNotFoundError.into());
        }

        let (frame_index, bitmap_pos) = start.unwrap();
        let start_frame_index = frame_index * BITMAP_SIZE + bitmap_pos;

        for i in start_frame_index..start_frame_index + count {
            self.alloc_frame(i)?;
        }

        let mem_frame_info = MemoryFrameInfo {
            frame_start_virt_addr: VirtualAddress::new(
                (start_frame_index * self.frame_size) as u64,
            ),
            frame_size: count * self.frame_size,
            frame_index: start_frame_index,
            is_allocated: true,
        };

        mem_frame_info.set_permissions(ReadWrite::Write, EntryMode::Supervisor)?;
        Ok(mem_frame_info)
    }

    pub fn mem_clear(&self, mem_frame_info: &MemoryFrameInfo) {
        let start_virt_addr = mem_frame_info.frame_start_virt_addr;
        let mut offset = 0;
        while offset < mem_frame_info.frame_size {
            start_virt_addr.offset(offset).write_volatile::<u64>(0);
            offset += size_of::<u64>();
        }
    }

    pub fn dealloc_mem_frame(&mut self, mem_frame_info: MemoryFrameInfo) -> Result<()> {
        let frame_size = mem_frame_info.frame_size;
        let frame_index = mem_frame_info.frame_index;

        for i in frame_index..frame_index + (frame_size + self.frame_size - 1) / self.frame_size {
            self.dealloc_frame(i)?;
        }

        Ok(())
    }

    fn get_mem_frame_index(&self, virt_addr: VirtualAddress) -> usize {
        virt_addr.get() as usize / self.frame_size
    }

    fn read_bitmap(&self, offset: usize) -> Option<Bitmap> {
        if offset >= self.bitmap_len {
            return None;
        }

        let addr = VirtualAddress::new(self.bitmap_virt_addr.get() + offset as u64);
        Some(Bitmap::new(addr.read_volatile()))
    }

    fn write_bitmap(&self, offset: usize, bitmap: Bitmap) -> Result<()> {
        if offset >= self.bitmap_len {
            return Err(BitmapMemoryManagerError::MemoryMapOffsetOutOfBounds(offset).into());
        }

        let addr = VirtualAddress::new(self.bitmap_virt_addr.get() + offset as u64);
        addr.write_volatile(bitmap.0);

        Ok(())
    }

    fn clear_bitmap(&self) {
        for i in 0..self.bitmap_len {
            let addr = VirtualAddress::new(self.bitmap_virt_addr.get() + i as u64);
            addr.write_volatile::<u8>(0);
        }
    }

    fn alloc_frame(&mut self, frame_index: usize) -> Result<()> {
        if frame_index >= self.frame_len {
            return Err(
                BitmapMemoryManagerError::MemoryFrameIndexOutOfBoundsError(frame_index).into(),
            );
        }

        let bitmap_offset = frame_index / BITMAP_SIZE;
        let bitmap_pos = frame_index % BITMAP_SIZE; // 0 ~ 7

        if let Some(bitmap) = self.read_bitmap(bitmap_offset) {
            let mut bitmap = bitmap;

            let mut map = bitmap.get_map();

            // already allocated
            if map[bitmap_pos] {
                return Err(
                    BitmapMemoryManagerError::MemoryFrameWasAlreadyAllocatedError(frame_index)
                        .into(),
                );
            }

            map[bitmap_pos] = true;
            bitmap.set_map(map);
            self.write_bitmap(bitmap_offset, bitmap)?;

            self.allocated_frame_len += 1;
            self.free_frame_len -= 1;

            return Ok(());
        }

        Err(BitmapMemoryManagerError::FailedToReadBitmapError.into())
    }

    fn dealloc_frame(&mut self, frame_index: usize) -> Result<()> {
        if frame_index >= self.frame_len {
            return Err(
                BitmapMemoryManagerError::MemoryFrameIndexOutOfBoundsError(frame_index).into(),
            );
        }

        let bitmap_offset = frame_index / BITMAP_SIZE;
        let bitmap_pos = frame_index % BITMAP_SIZE; // 0 ~ 7

        if let Some(bitmap) = self.read_bitmap(bitmap_offset) {
            let mut bitmap = bitmap;

            // already deallocated
            if !bitmap.get_map()[bitmap_pos] {
                return Err(
                    BitmapMemoryManagerError::MemoryFrameWasAlreadyDeallocatedError(frame_index)
                        .into(),
                );
            }

            let mut map = bitmap.get_map();
            map[bitmap_pos] = false;
            bitmap.set_map(map);
            self.write_bitmap(bitmap_offset, bitmap)?;

            self.allocated_frame_len -= 1;
            self.free_frame_len += 1;

            return Ok(());
        }

        Err(BitmapMemoryManagerError::FailedToReadBitmapError.into())
    }
}

pub fn init(mem_map: &[MemoryDescriptor]) -> Result<()> {
    if let Ok(mut bitmap_mem_man) = unsafe { BITMAP_MEM_MAN.try_lock() } {
        *bitmap_mem_man = match BitmapMemoryManager::new(mem_map) {
            Ok(bmm) => Some(bmm),
            Err(e) => return Err(e),
        };

        return Ok(());
    }

    Err(MutexError::Locked.into())
}

// (used, total)
pub fn get_mem_size() -> (usize, usize) {
    if let Ok(bitmap_mem_man) = unsafe { BITMAP_MEM_MAN.try_lock() } {
        if let Some(bitmap_mem_man) = bitmap_mem_man.as_ref() {
            return (
                bitmap_mem_man.get_used_mem_size(),
                bitmap_mem_man.get_total_mem_size(),
            );
        }
    }

    (0, 0)
}

pub fn alloc_mem_frame(len: usize) -> Result<MemoryFrameInfo> {
    if let Ok(mut bitmap_mem_man) = unsafe { BITMAP_MEM_MAN.try_lock() } {
        if let Some(bitmap_mem_man) = bitmap_mem_man.as_mut() {
            return bitmap_mem_man.alloc_multi_mem_frame(len);
        }

        return Err(BitmapMemoryManagerError::NotInitialized.into());
    }

    Err(MutexError::Locked.into())
}

pub fn dealloc_mem_frame(mem_frame_info: MemoryFrameInfo) -> Result<()> {
    if let Ok(mut bitmap_mem_man) = unsafe { BITMAP_MEM_MAN.try_lock() } {
        if let Some(bitmap_mem_man) = bitmap_mem_man.as_mut() {
            return bitmap_mem_man.dealloc_mem_frame(mem_frame_info);
        }

        return Err(BitmapMemoryManagerError::NotInitialized.into());
    }

    Err(MutexError::Locked.into())
}

pub fn mem_clear(mem_frame_info: &MemoryFrameInfo) -> Result<()> {
    if let Ok(bitmap_mem_man) = unsafe { BITMAP_MEM_MAN.try_lock() } {
        if let Some(bitmap_mem_man) = bitmap_mem_man.as_ref() {
            bitmap_mem_man.mem_clear(mem_frame_info);
            return Ok(());
        }

        return Err(BitmapMemoryManagerError::NotInitialized.into());
    }

    Err(MutexError::Locked.into())
}
