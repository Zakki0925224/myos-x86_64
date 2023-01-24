use core::ptr::{read_volatile, write_volatile};

use common::mem_desc::{MemoryDescriptor, MemoryType, UEFI_PAGE_SIZE};
use lazy_static::lazy_static;
use log::{info, warn};
use spin::Mutex;

use crate::{arch::addr::VirtualAddress, println};

lazy_static! {
    pub static ref BITMAP_MEM_MAN: Mutex<BitmapMemoryManager> =
        Mutex::new(BitmapMemoryManager::new());
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryFrameInfo
{
    frame_start_virt_addr: VirtualAddress,
    frame_size: usize,
    frame_index: usize,
    is_allocated: bool,
}

impl MemoryFrameInfo
{
    pub fn get_frame_start_virt_addr(&self) -> VirtualAddress { return self.frame_start_virt_addr; }

    pub fn get_frame_szie(&self) -> usize { return self.frame_size; }

    pub fn get_frame_index(&self) -> usize { return self.frame_index; }

    pub fn is_allocated(&self) -> bool { return self.is_allocated; }
}

#[derive(Debug)]
struct Bitmap(u8);

impl Bitmap
{
    pub fn new(bitmap: u8) -> Self { return Self(bitmap); }

    pub fn get_map(&self) -> [bool; u8::BITS as usize]
    {
        let mut map = [false; u8::BITS as usize];

        for i in 0..u8::BITS as usize
        {
            map[i] = (self.0 << 0) == 1;
        }

        return map;
    }

    pub fn allocated_frame_len(&self) -> usize
    {
        let mut len = 0;

        for i in 0..u8::BITS as usize
        {
            if (self.0 >> 8 - i) == 1
            {
                len += 1;
            }
        }

        return len;
    }

    pub fn free_frame_len(&self) -> usize
    {
        let mut len = 0;

        for i in 0..u8::BITS as usize
        {
            if (self.0 >> 8 - i) == 0
            {
                len += 1;
            }
        }

        return len;
    }

    pub fn is_allocated_all(&self) -> bool { return self.0 == 0xff; }

    pub fn is_free_all(&self) -> bool { return self.0 == 0; }
}

#[derive(Debug)]
pub struct BitmapMemoryManager
{
    is_init: bool,
    bitmap_virt_addr: VirtualAddress,
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
            bitmap_virt_addr: VirtualAddress::new(0),
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
        let mut bitmap_virt_addr = VirtualAddress::new(0);
        for d in mem_map
        {
            if d.ty != MemoryType::Conventional
                || d.page_cnt as usize * UEFI_PAGE_SIZE < bitmap_size
                || d.phys_start == 0
            {
                continue;
            }

            bitmap_virt_addr.set(d.phys_start);
            break;
        }

        if bitmap_virt_addr.get() == 0
        {
            panic!("Failed to find available memory area for bitmap");
        }

        self.bitmap_virt_addr = bitmap_virt_addr;
        self.bitmap_size = bitmap_size;
        self.frame_len = total_page_cnt;
        self.allocated_frame_len = 0;
        self.free_frame_len = self.frame_len;
        self.frame_size = UEFI_PAGE_SIZE;

        self.is_init = true;

        // clear all bitmap
        for i in 0..self.bitmap_size
        {
            self.write_bitmap(i, Bitmap(0));
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
        let start = self.get_mem_frame_index(self.bitmap_virt_addr);
        let end = self.get_mem_frame_index(self.bitmap_virt_addr.offset(self.bitmap_size));
        for i in start..=end
        {
            self.alloc_frame(i);
        }

        // allocate less 1MB memory space
        let start = 0;
        let end = self.get_mem_frame_index(VirtualAddress::new(1024 * 1024));
        for i in start..=end
        {
            self.alloc_frame(i);
        }

        info!("Initialized bitmap memory manager");
    }

    pub fn is_init(&self) -> bool { return self.is_init; }

    pub fn get_frame_size(&self) -> usize { return self.frame_size; }

    pub fn get_total_mem_size(&self) -> usize { return self.frame_size * self.frame_len; }

    pub fn get_used_mem_size(&self) -> usize { return self.allocated_frame_len * self.frame_size; }

    pub fn get_mem_frame(&self, frame_index: usize) -> Option<MemoryFrameInfo>
    {
        if !self.is_init || frame_index >= self.frame_len
        {
            return None;
        }

        let bitmap_offset = frame_index / u8::BITS as usize;
        let bitmap_pos = frame_index % u8::BITS as usize; // 0 ~ 7
        let bitmap = self.read_bitmap(bitmap_offset);

        return Some(MemoryFrameInfo {
            frame_start_virt_addr: VirtualAddress::new((frame_index * self.frame_size) as u64),
            frame_size: self.frame_size,
            frame_index,
            is_allocated: bitmap.get_map()[bitmap_pos],
        });
    }

    pub fn alloc_single_mem_frame(&mut self) -> Option<MemoryFrameInfo>
    {
        if self.free_frame_len == 0
        {
            return None;
        }

        let mut found_mem_frame_index = None;
        'outer: for i in 0..self.bitmap_size
        {
            let bitmap = self.read_bitmap(i);

            if bitmap.is_allocated_all()
            {
                continue;
            }

            for j in 0..u8::BITS as usize
            {
                if bitmap.get_map()[j]
                {
                    found_mem_frame_index = Some(i * u8::BITS as usize + j);
                    break 'outer;
                }
            }
        }

        if let None = found_mem_frame_index
        {
            return None;
        }

        let mem_frame_info = MemoryFrameInfo {
            frame_start_virt_addr: VirtualAddress::new(
                (found_mem_frame_index.unwrap() * self.frame_size) as u64,
            ),
            frame_size: self.frame_size,
            frame_index: found_mem_frame_index.unwrap(),
            is_allocated: true,
        };

        self.alloc_frame(found_mem_frame_index.unwrap());
        //self.mem_clear(&mem_frame_info);

        return Some(mem_frame_info);
    }

    pub fn alloc_multi_mem_frame(&mut self, len: usize) -> Option<MemoryFrameInfo>
    {
        if len > u8::BITS as usize
        {
            warn!(
                "Unsupported memory frame length (max: {}, but specified: {})",
                u8::BITS as usize,
                len
            );
            return None;
        }

        if len == 0 || self.free_frame_len < len
        {
            return None;
        }

        if len == 1
        {
            return self.alloc_single_mem_frame();
        }

        let mut allocated_cnt = 0;
        let mut i = 0;

        while i < self.bitmap_size
        {
            let bitmap = self.read_bitmap(i);

            i += 1;
        }

        return None;
    }

    pub fn mem_clear(&self, mem_frame_info: &MemoryFrameInfo)
    {
        for i in mem_frame_info.frame_start_virt_addr.get()
            ..mem_frame_info.frame_start_virt_addr.get() + mem_frame_info.frame_size as u64
        {
            let ptr = i as *mut u8;
            unsafe { write_volatile(ptr, 0) };
        }
    }

    pub fn dealloc_mem_frame(&mut self, mem_frame_info: MemoryFrameInfo)
    {
        self.mem_clear(&mem_frame_info);
        self.dealloc_frame(mem_frame_info.frame_index);
    }

    fn get_mem_frame_index(&self, virt_addr: VirtualAddress) -> usize
    {
        let index = virt_addr.get() as usize / self.frame_size;

        return index;
    }

    fn read_bitmap(&self, offset: usize) -> Bitmap
    {
        if !self.is_init
        {
            panic!("Bitmap memory manager was not initialized");
        }

        if offset >= self.bitmap_size
        {
            panic!("Memory map offset out of bounds");
        }

        let ptr = (self.bitmap_virt_addr.get() + offset as u64) as *const u8;
        return Bitmap::new(unsafe { read_volatile(ptr) });
    }

    fn write_bitmap(&self, offset: usize, bitmap: Bitmap)
    {
        if !self.is_init
        {
            panic!("Bitmap memory manager was not initialized");
        }

        if offset >= self.bitmap_size
        {
            panic!("Memory map offset out of bounds");
        }

        let ptr = (self.bitmap_virt_addr.get() + offset as u64) as *mut u8;
        unsafe { write_volatile(ptr, bitmap.0) };
    }

    fn alloc_frame(&mut self, frame_index: usize)
    {
        if frame_index >= self.frame_len
        {
            panic!("Memory frame index out of bounds");
        }

        let bitmap_offset = frame_index / u8::BITS as usize;
        let bitmap_pos = frame_index % u8::BITS as usize; // 0 ~ 7

        let mut bitmap = self.read_bitmap(bitmap_offset);

        // already allocated
        if bitmap.get_map()[bitmap_pos]
        {
            return;
        }

        bitmap.0 |= 0x80 >> bitmap_pos;
        self.write_bitmap(bitmap_offset, bitmap);

        self.allocated_frame_len += 1;
        self.free_frame_len -= 1;
    }

    fn dealloc_frame(&mut self, frame_index: usize)
    {
        if frame_index >= self.frame_len
        {
            panic!("Memory frame index out of bounds");
        }

        let bitmap_offset = frame_index / u8::BITS as usize;
        let bitmap_pos = frame_index % u8::BITS as usize; // 0 ~ 7

        let mut bitmap = self.read_bitmap(bitmap_offset);

        if !bitmap.get_map()[bitmap_pos]
        {
            panic!("This memory frame was already deallocated");
        }

        bitmap.0 &= !(0x80 >> bitmap_pos);
        self.write_bitmap(bitmap_offset, bitmap);

        self.allocated_frame_len -= 1;
        self.free_frame_len += 1;
    }
}
