use common::mem_desc::UEFI_PAGE_SIZE;
use core::{alloc::{GlobalAlloc, Layout}, cell::UnsafeCell, ptr::null_mut};
use log::info;

use super::bitmap::BITMAP_MEM_MAN;

const HEAP_SIZE: usize = UEFI_PAGE_SIZE * UEFI_PAGE_SIZE; // 16MiB

#[global_allocator]
pub static ALLOCATOR: Allocator =
    Allocator { start_virt_addr: UnsafeCell::new(0), base_virt_addr: UnsafeCell::new(0) };

pub struct Allocator
{
    pub start_virt_addr: UnsafeCell<u64>,
    base_virt_addr: UnsafeCell<u64>,
}

unsafe impl Sync for Allocator {}

unsafe impl GlobalAlloc for Allocator
{
    unsafe fn alloc(&self, layout: Layout) -> *mut u8
    {
        let size = layout.size() as u64;
        let align = layout.align() as u64;
        let offset = (size + (align - 1)) & !(align - 1);
        let start_virt_addr = self.start_virt_addr.get();
        let base_virt_addr = self.base_virt_addr.get();
        let base_virt_addr_clone = (*base_virt_addr).clone();

        if *start_virt_addr == 0 && *base_virt_addr == 0
        {
            panic!("Heap was used before heap allocator was initialized");
        }

        if size > HEAP_SIZE as u64
        {
            return null_mut();
        }

        if align > size
        {
            return null_mut();
        }

        if *base_virt_addr + offset > *start_virt_addr + HEAP_SIZE as u64
        {
            return null_mut();
        }

        *base_virt_addr += offset;
        return base_virt_addr_clone as *mut u8;
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

impl Allocator
{
    pub fn init(&self)
    {
        // allocate for heap area
        if let Ok(mem_frame_info) =
            BITMAP_MEM_MAN.lock().alloc_multi_mem_frame(HEAP_SIZE / UEFI_PAGE_SIZE)
        {
            let virt_addr = mem_frame_info.get_frame_start_virt_addr();
            unsafe {
                (*self.start_virt_addr.get()) = virt_addr.get();
                (*self.base_virt_addr.get()) = virt_addr.get();
            }

            info!("mem: Initialized heap allocator");
        }
        else
        {
            panic!("mem: Failed to allocate heap area");
        }
    }
}
