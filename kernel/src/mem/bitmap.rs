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
    frame_size: usize, // must be 4096B align
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

const BITMAP_SIZE: usize = u8::BITS as usize;
#[derive(Debug)]
struct Bitmap(u8);

impl Bitmap
{
    pub fn new(bitmap: u8) -> Self { return Self(bitmap); }

    pub fn get_map(&self) -> [bool; BITMAP_SIZE]
    {
        let mut map = [false; BITMAP_SIZE];

        for i in 0..BITMAP_SIZE
        {
            map[i] = ((self.0 << i) & 0x80) != 0;
        }

        //println!("read: {:b}", self.0);
        //println!("read: {:?}", map);

        return map;
    }

    pub fn set_map(&mut self, map: [bool; BITMAP_SIZE])
    {
        //println!("new map: {:?}", map);
        let mut bitmap = 0;
        for i in 0..BITMAP_SIZE
        {
            bitmap |= (map[i] as u8) << BITMAP_SIZE - 1 - i;
        }

        self.0 = bitmap;
        //println!("new bitmap: {:b}", self.0);
    }

    pub fn allocated_frame_len(&self) -> usize
    {
        let mut len = 0;
        let map = self.get_map();

        for i in 0..BITMAP_SIZE
        {
            if map[i]
            {
                len += 1;
            }
        }

        return len;
    }

    pub fn free_frame_len(&self) -> usize
    {
        let mut len = 0;
        let map = self.get_map();

        for i in 0..BITMAP_SIZE
        {
            if !map[i]
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
    bitmap_len: usize,
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
            bitmap_len: 0,
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
        // get bitmap len
        let bitmap_len = total_page_cnt / BITMAP_SIZE;

        // find available memory area for bitmap
        let mut bitmap_virt_addr = VirtualAddress::new(0);
        for d in mem_map
        {
            if d.ty != MemoryType::Conventional
                || d.page_cnt as usize * UEFI_PAGE_SIZE < bitmap_len
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
        self.bitmap_len = bitmap_len;
        self.frame_len = total_page_cnt;
        self.allocated_frame_len = 0;
        self.free_frame_len = self.frame_len;
        self.frame_size = UEFI_PAGE_SIZE;

        self.is_init = true;

        // clear all bitmap
        self.clear_bitmap();

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
        let end = self.get_mem_frame_index(self.bitmap_virt_addr.offset(self.bitmap_len));
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

        let bitmap_offset = frame_index / BITMAP_SIZE;
        let bitmap_pos = frame_index % BITMAP_SIZE; // 0 ~ 7
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
        'outer: for i in 0..self.bitmap_len
        {
            let bitmap = self.read_bitmap(i);

            if bitmap.is_allocated_all()
            {
                continue;
            }

            for j in 0..BITMAP_SIZE
            {
                if !bitmap.get_map()[j]
                {
                    found_mem_frame_index = Some(i * BITMAP_SIZE + j);
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
        if len == 0 || self.free_frame_len < len
        {
            return None;
        }

        if len == 1
        {
            return self.alloc_single_mem_frame();
        }

        let mut start = None;
        let mut count = 0;
        let mut i = 0;

        'outer: while i < self.bitmap_len
        {
            let bitmap = self.read_bitmap(i);

            let map = bitmap.get_map();
            //println!("{}: {:?}", i, map);
            //println!("{:b}", bitmap.0);
            for j in 0..BITMAP_SIZE
            {
                if start != None && count == len
                {
                    break 'outer;
                }

                if len - count > BITMAP_SIZE && bitmap.is_free_all() && start != None
                {
                    count += BITMAP_SIZE;
                    //println!("skip with alloc ({})", count);
                    break;
                }

                if len - count > BITMAP_SIZE && bitmap.is_allocated_all()
                {
                    start = None;
                    count = 0;
                    //println!("reset ({}, all)", i);
                    break;
                }

                // free
                if !map[j]
                {
                    count += 1;
                    //println!("new count: {}", count);

                    if start == None
                    {
                        start = Some((i, j));
                    }
                }
                // already allocated
                else
                {
                    start = None;
                    count = 0;
                }
            }

            i += 1;
        }

        if start == None || count != len
        {
            return None;
        }

        let (frame_index, bitmap_pos) = start.unwrap();
        let start_frame_index = frame_index * BITMAP_SIZE + bitmap_pos;

        for i in start_frame_index..start_frame_index + count
        {
            self.alloc_frame(i);
        }

        let mem_frame_info = MemoryFrameInfo {
            frame_start_virt_addr: VirtualAddress::new(
                (start_frame_index * self.frame_size) as u64,
            ),
            frame_size: count * self.frame_size,
            frame_index: start_frame_index,
            is_allocated: true,
        };

        return Some(mem_frame_info);
    }

    pub fn mem_clear(&self, mem_frame_info: &MemoryFrameInfo)
    {
        let start_virt_addr = mem_frame_info.frame_start_virt_addr.get();
        for i in start_virt_addr..start_virt_addr + mem_frame_info.frame_size as u64
        {
            let ptr = i as *mut u8;
            unsafe { write_volatile(ptr, 0) };
        }
    }

    pub fn dealloc_mem_frame(&mut self, mem_frame_info: MemoryFrameInfo)
    {
        self.mem_clear(&mem_frame_info);

        let frame_size = mem_frame_info.frame_size;
        if frame_size == self.frame_size
        {
            self.dealloc_frame(mem_frame_info.frame_index);
            return;
        }

        let frame_index = mem_frame_info.frame_index;
        for i in frame_index..frame_index + (frame_size + self.frame_size - 1) / self.frame_size
        {
            self.dealloc_frame(i);
        }
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

        if offset >= self.bitmap_len
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

        if offset >= self.bitmap_len
        {
            panic!("Memory map offset out of bounds");
        }

        let ptr = (self.bitmap_virt_addr.get() + offset as u64) as *mut u8;
        unsafe { write_volatile(ptr, bitmap.0) };
        let read = unsafe { read_volatile(ptr) };
    }

    fn clear_bitmap(&self)
    {
        for i in 0..self.bitmap_len
        {
            let ptr = (self.bitmap_virt_addr.get() + i as u64) as *mut u8;
            unsafe { write_volatile(ptr, 0) };
        }
    }

    fn alloc_frame(&mut self, frame_index: usize)
    {
        if frame_index >= self.frame_len
        {
            panic!("Memory frame index out of bounds");
        }

        let bitmap_offset = frame_index / BITMAP_SIZE;
        let bitmap_pos = frame_index % BITMAP_SIZE; // 0 ~ 7

        let mut bitmap = self.read_bitmap(bitmap_offset);
        let mut map = bitmap.get_map();

        // already allocated
        if map[bitmap_pos]
        {
            return;
        }

        map[bitmap_pos] = true;
        bitmap.set_map(map);
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

        let bitmap_offset = frame_index / BITMAP_SIZE;
        let bitmap_pos = frame_index % BITMAP_SIZE; // 0 ~ 7

        let mut bitmap = self.read_bitmap(bitmap_offset);

        if !bitmap.get_map()[bitmap_pos]
        {
            panic!("This memory frame was already deallocated");
        }

        let mut map = bitmap.get_map();
        map[bitmap_pos] = false;
        bitmap.set_map(map);
        self.write_bitmap(bitmap_offset, bitmap);

        self.allocated_frame_len -= 1;
        self.free_frame_len += 1;
    }
}
