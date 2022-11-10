use core::{alloc::{GlobalAlloc, Layout}, cell::UnsafeCell, ptr::null_mut};

const HEAP_SIZE: usize = 1024 * 1024 * 1024; // 10MiB
const HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

#[global_allocator]
static ALLOCATOR: Allocator =
    Allocator { base_addr: UnsafeCell::new(unsafe { HEAP.as_ptr() as u64 }) };

pub struct Allocator
{
    base_addr: UnsafeCell<u64>,
}

unsafe impl Sync for Allocator {}

unsafe impl GlobalAlloc for Allocator
{
    unsafe fn alloc(&self, layout: Layout) -> *mut u8
    {
        let size = layout.size();
        let align = layout.align();
        let base_addr = self.base_addr.get();
        let heap_base_addr = unsafe { HEAP.as_ptr() as u64 };

        if size > HEAP_SIZE
        {
            return null_mut();
        }

        if align > size
        {
            return null_mut();
        }

        let offset = ((size + (align - 1)) & !(align - 1)) as u64;

        if *base_addr + offset > heap_base_addr + HEAP_SIZE as u64
        {
            return null_mut();
        }

        let before_base_addr = (*self.base_addr.get()).clone();
        *base_addr += offset;

        return before_base_addr as *mut u8;
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}
