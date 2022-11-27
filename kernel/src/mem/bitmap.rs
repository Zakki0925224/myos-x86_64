use core::ptr::{read_volatile, write_volatile};

use common::mem_desc::{MemoryDescriptor, MemoryType, UEFI_PAGE_SIZE};
use lazy_static::lazy_static;
use log::info;
use spin::Mutex;

lazy_static! {
    pub static ref BITMAP_MEM_MAN: Mutex<BitmapMemoryManager> =
        Mutex::new(BitmapMemoryManager::new());
}

#[derive(Debug)]
pub struct BitmapMemoryManager
{
    is_init: bool,
    bitmap_base_addr: u64,
    bitmap_size: usize,
    frame_len: usize,
    allocated_frame_len: usize,
    free_frame_len: usize,
    frame_size: usize,
}

impl BitmapMemoryManager
{
    pub fn new() -> Self
    {
        return Self {
            is_init: false,
            bitmap_base_addr: 0,
            bitmap_size: 0,
            frame_len: 0,
            allocated_frame_len: 0,
            free_frame_len: 0,
            frame_size: 0,
        };
    }

    pub fn init(&mut self, mem_map: &[MemoryDescriptor])
    {
        // TODO: boot services data/code

        // get total page count (a page=4096B)
        let total_page_cnt = mem_map.into_iter().map(|d| d.page_cnt as usize).sum();
        // get bitmap size
        let bitmap_size = total_page_cnt / u8::BITS as usize;

        // find available memory area for bitmap
        let mut bitmap_base_addr = 0;
        for d in mem_map
        {
            if d.ty != MemoryType::Conventional
                || d.page_cnt as usize * UEFI_PAGE_SIZE < bitmap_size
                || d.phys_start == 0
            {
                continue;
            }

            bitmap_base_addr = d.phys_start;
            break;
        }

        if bitmap_base_addr == 0
        {
            panic!("Failed to find available memory area for bitmap");
        }

        self.bitmap_base_addr = bitmap_base_addr;
        self.bitmap_size = bitmap_size;
        self.frame_len = total_page_cnt;
        self.allocated_frame_len = 0;
        self.free_frame_len = self.frame_len;
        self.frame_size = UEFI_PAGE_SIZE;

        self.is_init = true;

        // clear all bitmap
        for i in 0..self.bitmap_size
        {
            self.write_bitmap(i, 0);
        }

        // allocate no conventional memory frame
        let mut frame_index = 0;

        for d in mem_map
        {
            if d.ty == MemoryType::Conventional
            {
                frame_index += d.page_cnt as usize;
                continue;
            }

            for _ in 0..d.page_cnt
            {
                self.alloc_frame(frame_index);
                frame_index += 1;
            }
        }

        // allocate bitmap memory frame
        let start = self.get_mem_frame_index(self.bitmap_base_addr);
        let end = self.get_mem_frame_index(self.bitmap_base_addr + self.bitmap_size as u64);
        for i in start..=end
        {
            self.alloc_frame(i);
        }

        info!("Initialized bitmap memory manager");
    }

    pub fn is_init(&self) -> bool { return self.is_init; }

    pub fn get_total_mem_size(&self) -> usize { return self.frame_size * self.frame_len; }

    pub fn get_used_mem_size(&self) -> usize { return self.allocated_frame_len * self.frame_size; }

    fn get_mem_frame_index(&self, base_addr: u64) -> usize
    {
        let index = base_addr as usize / self.frame_size;

        return index;
    }

    fn read_bitmap(&self, offset: usize) -> u8
    {
        if !self.is_init
        {
            panic!("Bitmap memory manager was not initialized");
        }

        if offset >= self.bitmap_size
        {
            panic!("Memory map offset out of bounds");
        }

        let ptr = (self.bitmap_base_addr + offset as u64) as *const u8;
        return unsafe { read_volatile(ptr) };
    }

    fn write_bitmap(&self, offset: usize, bitmap: u8)
    {
        if !self.is_init
        {
            panic!("Bitmap memory manager was not initialized");
        }

        if offset >= self.bitmap_size
        {
            panic!("Memory map offset out of bounds");
        }

        let ptr = (self.bitmap_base_addr + offset as u64) as *mut u8;
        unsafe { write_volatile(ptr, bitmap) };
    }

    fn alloc_frame(&mut self, frame_index: usize)
    {
        if !self.is_init
        {
            panic!("Bitmap memory manager was not initialized");
        }

        if frame_index >= self.frame_len
        {
            panic!("Memory frame index out of bounds");
        }

        let bitmap_offset = frame_index / u8::BITS as usize;
        let bitmap_pos = frame_index % u8::BITS as usize; // 0 ~ 7

        let mut bitmap = self.read_bitmap(bitmap_offset);

        if (bitmap << bitmap_pos) & 0x80 == 0x80
        {
            panic!("This memory frame was already allocated");
        }

        bitmap |= 0x80 >> bitmap_pos;
        self.write_bitmap(bitmap_offset, bitmap);

        self.allocated_frame_len += 1;
        self.free_frame_len -= 1;
    }

    fn dealloc_frame(&mut self, frame_index: usize)
    {
        if !self.is_init
        {
            panic!("Bitmap memory manager was not initialized");
        }

        if frame_index >= self.frame_len
        {
            panic!("Memory frame index out of bounds");
        }

        let bitmap_offset = frame_index / u8::BITS as usize;
        let bitmap_pos = frame_index % u8::BITS as usize; // 0 ~ 7

        let mut bitmap = self.read_bitmap(bitmap_offset);

        if (bitmap >> bitmap_pos) & 0x80 == 0
        {
            panic!("This memory frame was already deallocated");
        }

        bitmap &= !(0x80 >> bitmap_pos);
        self.write_bitmap(bitmap_offset, bitmap);

        self.allocated_frame_len -= 1;
        self.free_frame_len += 1;
    }
}
