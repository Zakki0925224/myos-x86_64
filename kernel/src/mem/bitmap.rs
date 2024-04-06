use super::paging::{self, EntryMode, ReadWrite, PAGE_SIZE};
use crate::{
    arch::addr::*,
    error::{Error, Result},
    util::mutex::{Mutex, MutexError},
};
use alloc::vec::Vec;
use common::mem_desc::*;

static mut BITMAP_MEM_MAN: Mutex<Option<BitmapMemoryManager>> = Mutex::new(None);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MemoryFrameInfo {
    pub frame_start_phys_addr: PhysicalAddress,
    pub frame_size: usize, // must be 4096B align
    pub frame_index: usize,
    pub is_allocated: bool,
}

impl MemoryFrameInfo {
    pub fn set_permissions_to_supervisor(&self) -> Result<()> {
        self.set_permissions(ReadWrite::Write, EntryMode::Supervisor)
    }

    pub fn set_permissions_to_user(&self) -> Result<()> {
        self.set_permissions(ReadWrite::Write, EntryMode::User)
    }

    pub fn frame_start_virt_addr(&self) -> Result<VirtualAddress> {
        self.frame_start_phys_addr.get_virt_addr()
    }

    pub fn set_permissions(&self, rw: ReadWrite, mode: EntryMode) -> Result<()> {
        let page_len = self.frame_size / PAGE_SIZE;
        let mut start_virt_addr = self.frame_start_virt_addr()?;

        for _ in 0..page_len {
            paging::set_page_permissions(start_virt_addr, rw, mode)?;
            start_virt_addr = start_virt_addr.offset(PAGE_SIZE);
        }

        Ok(())
    }

    pub fn get_permissions(&self) -> Result<Vec<(ReadWrite, EntryMode)>> {
        let page_len = self.frame_size / PAGE_SIZE;
        let mut start_virt_addr = self.frame_start_virt_addr()?;
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
    pub fn get(&self, index: usize) -> Result<bool> {
        if index >= BITMAP_SIZE {
            return Err(Error::IndexOutOfBoundsError(index));
        }

        Ok(((self.0 >> index) & 0x1) != 0)
    }

    pub fn set(&mut self, index: usize, value: bool) -> Result<()> {
        if index >= BITMAP_SIZE {
            return Err(Error::IndexOutOfBoundsError(index));
        }

        self.0 = (self.0 & !(0x1 << index)) | ((value as u8) << index);
        assert_eq!(self.get(index)?, value);

        Ok(())
    }

    pub fn fill(&mut self, value: bool) {
        self.0 = if value { 0xff } else { 0 };
    }

    pub fn is_allocated_all(&self) -> bool {
        self.0 == 0xff
    }

    pub fn is_free_all(&self) -> bool {
        self.0 == 0
    }
}

impl From<u8> for Bitmap {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitmapMemoryManagerError {
    NotInitialized,
    FreeMemoryFrameWasNotFoundError,
    MemoryFrameWasAlreadyAllocatedError(usize), // memory frame index,
    MemoryFrameWasAlreadyDeallocatedError(usize), // memory frame index,
    InvalidMemoryFrameLengthError(usize),       // memory frame length
}

#[derive(Debug)]
pub struct BitmapMemoryManager {
    bitmap_phys_addr: PhysicalAddress,
    bitmap_len: usize,
    frame_len: usize,
    allocated_frame_len: usize,
    free_frame_len: usize,
    frame_size: usize,
}

impl BitmapMemoryManager {
    pub fn new(mem_map: &[MemoryDescriptor]) -> Self {
        // TODO: boot services data/code
        let max_phys_addr = mem_map.iter().map(|d| d.phys_start).max().unwrap();
        let mem_size = mem_map
            .iter()
            .find(|d| d.phys_start == max_phys_addr)
            .unwrap()
            .page_cnt
            * UEFI_PAGE_SIZE as u64;
        let total_page_cnt = (max_phys_addr + mem_size) as usize / UEFI_PAGE_SIZE;
        let bitmap_len = total_page_cnt / BITMAP_SIZE;

        // find available memory area for bitmap
        let mut bitmap_phys_addr = PhysicalAddress::default();
        for d in mem_map {
            if d.ty != MemoryType::Conventional
                || (d.page_cnt as usize) * UEFI_PAGE_SIZE < bitmap_len
                || d.phys_start == 0
                || d.phys_start % UEFI_PAGE_SIZE as u64 != 0
            {
                continue;
            }

            bitmap_phys_addr.set(d.phys_start);
            break;
        }

        if bitmap_phys_addr.get() == 0 {
            panic!("mem: Failed to allocate memory for bitmap");
        }

        Self {
            bitmap_phys_addr,
            bitmap_len,
            frame_len: total_page_cnt,
            allocated_frame_len: total_page_cnt,
            free_frame_len: 0,
            frame_size: UEFI_PAGE_SIZE,
        }
    }

    pub fn init(&mut self, mem_map: &[MemoryDescriptor]) -> Result<()> {
        // fill all bitmap
        for i in 0..self.bitmap_len {
            self.bitmap(i)?.fill(true);
        }

        // deallocate available memory frame
        for d in mem_map {
            match d.ty {
                MemoryType::BootServicesCode |
                /*MemoryType::BootServicesData |*/ // uefi stack, gdt, idt, page table
                MemoryType::Conventional => (),
                _ => continue,
            }

            if d.phys_start == 0 {
                continue;
            }

            if d.phys_start % (self.frame_size as u64) != 0 {
                continue;
            }

            for i in 0..d.page_cnt {
                let frame_index =
                    (d.phys_start + (i * self.frame_size as u64)) as usize / self.frame_size;

                self.dealloc_frame(frame_index)?;
            }
        }

        // allocate bitmap memory frame
        let start = self.bitmap_phys_addr.get() as usize / self.frame_size;
        let end = self.bitmap_phys_addr.offset(self.bitmap_len).get() as usize / self.frame_size;
        for i in start..=end {
            // ignore already allocated error
            let _ = self.alloc_frame(i);
        }

        Ok(())
    }

    pub fn get_total_mem_size(&self) -> usize {
        self.frame_size * self.frame_len
    }

    pub fn get_used_mem_size(&self) -> usize {
        self.allocated_frame_len * self.frame_size
    }

    pub fn get_mem_frame(&self, frame_index: usize) -> Option<MemoryFrameInfo> {
        if let Ok(bitmap) = self.bitmap(self.bitmap_offset(frame_index)) {
            return Some(MemoryFrameInfo {
                frame_start_phys_addr: ((frame_index * self.frame_size) as u64).into(),
                frame_size: self.frame_size,
                frame_index,
                is_allocated: bitmap.get(self.bitmap_pos(frame_index)).unwrap(),
            });
        }

        None
    }

    pub fn alloc_single_mem_frame(&mut self) -> Result<MemoryFrameInfo> {
        if self.free_frame_len == 0 {
            return Err(BitmapMemoryManagerError::FreeMemoryFrameWasNotFoundError.into());
        }

        let mut found_mem_frame_index = 0;
        'outer: for i in 0..self.bitmap_len {
            let bitmap = self.bitmap(i)?;
            if bitmap.is_allocated_all() {
                continue;
            }

            for j in 0..BITMAP_SIZE {
                if !bitmap.get(j)? {
                    found_mem_frame_index = i * BITMAP_SIZE + j;
                    break 'outer;
                }
            }
        }

        assert_ne!(found_mem_frame_index, 0);
        self.alloc_frame(found_mem_frame_index)?;
        let mem_frame_info = MemoryFrameInfo {
            frame_start_phys_addr: ((found_mem_frame_index * self.frame_size) as u64).into(),
            frame_size: self.frame_size,
            frame_index: found_mem_frame_index,
            is_allocated: true,
        };

        Ok(mem_frame_info)
    }

    pub fn alloc_multi_mem_frame(&mut self, len: usize) -> Result<MemoryFrameInfo> {
        if len == 0 {
            return Err(BitmapMemoryManagerError::InvalidMemoryFrameLengthError(len).into());
        }

        if len == 1 {
            return self.alloc_single_mem_frame();
        }

        if len > self.free_frame_len {
            return Err(BitmapMemoryManagerError::FreeMemoryFrameWasNotFoundError.into());
        }

        let mut start_mem_frame_index = None;
        let mut end_mem_frame_index = None;

        'outer: for i in 0..self.bitmap_len {
            let bitmap = self.bitmap(i)?;

            if len == BITMAP_SIZE && bitmap.is_free_all() {
                start_mem_frame_index = Some(i * BITMAP_SIZE);
                end_mem_frame_index = Some(i * BITMAP_SIZE + 7);
                break 'outer;
            }

            for j in 0..BITMAP_SIZE {
                // found all free area
                if let (Some(s_i), Some(e_i)) = (start_mem_frame_index, end_mem_frame_index) {
                    if e_i == s_i + len {
                        break 'outer;
                    }
                }

                if !bitmap.get(j)? {
                    if let Some(s_i) = start_mem_frame_index {
                        if let Some(e_i) = end_mem_frame_index.as_mut() {
                            *e_i += 1;
                        } else {
                            end_mem_frame_index = Some(s_i + 1);
                        }
                    } else {
                        start_mem_frame_index = Some(i * BITMAP_SIZE + j);
                    }
                } else {
                    start_mem_frame_index = None;
                    end_mem_frame_index = None;
                }
            }
        }

        let start_mem_frame_index = start_mem_frame_index.unwrap();
        let end_mem_frame_index = end_mem_frame_index.unwrap();

        for i in start_mem_frame_index..=end_mem_frame_index {
            self.alloc_frame(i)?;
        }

        let mem_frame_info = MemoryFrameInfo {
            frame_start_phys_addr: ((start_mem_frame_index * self.frame_size) as u64).into(),
            frame_size: self.frame_size * len,
            frame_index: start_mem_frame_index,
            is_allocated: true,
        };

        Ok(mem_frame_info)
    }

    pub unsafe fn mem_clear(&self, mem_frame_info: &MemoryFrameInfo) -> Result<()> {
        let frame_size = mem_frame_info.frame_size;
        let start_virt_addr = mem_frame_info.frame_start_virt_addr()?;

        // TODO: replace to other methods
        for offset in (0..frame_size).step_by(8) {
            let ref_value = start_virt_addr.offset(offset).as_ptr_mut() as *mut u64;
            *ref_value = 0;
        }

        Ok(())
    }

    pub fn dealloc_mem_frame(&mut self, mem_frame_info: MemoryFrameInfo) -> Result<()> {
        let frame_size = mem_frame_info.frame_size;
        let frame_index = mem_frame_info.frame_index;

        for i in frame_index..frame_index + (frame_size / self.frame_size) {
            self.dealloc_frame(i)?;
        }

        Ok(())
    }

    fn bitmap(&self, offset: usize) -> Result<&mut Bitmap> {
        if offset >= self.bitmap_len {
            return Err(Error::IndexOutOfBoundsError(offset));
        }

        Ok(unsafe { &mut *(self.bitmap_phys_addr.offset(offset).get() as *mut Bitmap) })
    }

    fn alloc_frame(&mut self, frame_index: usize) -> Result<()> {
        let bitmap_pos = self.bitmap_pos(frame_index);
        let bitmap = self.bitmap(self.bitmap_offset(frame_index))?;

        // already allocated
        if bitmap.get(bitmap_pos)? {
            return Err(
                BitmapMemoryManagerError::MemoryFrameWasAlreadyAllocatedError(frame_index).into(),
            );
        }

        bitmap.set(bitmap_pos, true)?;

        self.allocated_frame_len += 1;
        self.free_frame_len -= 1;

        Ok(())
    }

    fn dealloc_frame(&mut self, frame_index: usize) -> Result<()> {
        let bitmap_pos = self.bitmap_pos(frame_index);
        let bitmap = self.bitmap(self.bitmap_offset(frame_index))?;

        // already deallocated
        if !bitmap.get(bitmap_pos)? {
            return Err(
                BitmapMemoryManagerError::MemoryFrameWasAlreadyDeallocatedError(frame_index).into(),
            );
        }

        bitmap.set(bitmap_pos, false)?;

        self.allocated_frame_len -= 1;
        self.free_frame_len += 1;

        Ok(())
    }

    fn bitmap_offset(&self, frame_index: usize) -> usize {
        frame_index / BITMAP_SIZE
    }

    fn bitmap_pos(&self, frame_index: usize) -> usize {
        frame_index % BITMAP_SIZE // 0 ~ 7
    }
}

pub fn init(mem_map: &[MemoryDescriptor]) -> Result<()> {
    if let Ok(mut bitmap_mem_man) = unsafe { BITMAP_MEM_MAN.try_lock() } {
        let mut bmm = BitmapMemoryManager::new(mem_map);
        bmm.init(mem_map)?;
        *bitmap_mem_man = Some(bmm);

        return Ok(());
    }

    Err(MutexError::Locked.into())
}

// (used, total)
pub fn get_mem_size() -> Result<(usize, usize)> {
    if let Ok(bitmap_mem_man) = unsafe { BITMAP_MEM_MAN.try_lock() } {
        if let Some(bitmap_mem_man) = bitmap_mem_man.as_ref() {
            return Ok((
                bitmap_mem_man.get_used_mem_size(),
                bitmap_mem_man.get_total_mem_size(),
            ));
        }
    }

    Err(MutexError::Locked.into())
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
    unsafe {
        if let Ok(bitmap_mem_man) = BITMAP_MEM_MAN.try_lock() {
            if let Some(bitmap_mem_man) = bitmap_mem_man.as_ref() {
                return bitmap_mem_man.mem_clear(mem_frame_info);
            }

            return Err(BitmapMemoryManagerError::NotInitialized.into());
        }
    }

    Err(MutexError::Locked.into())
}
