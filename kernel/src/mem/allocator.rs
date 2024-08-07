use super::{bitmap, paging::PAGE_SIZE};
use crate::{error::Result, util::mutex::MutexError};
use alloc::{string::String, vec::Vec};
use core::alloc::Layout;
use linked_list_allocator::LockedHeap;

const HEAP_SIZE: usize = PAGE_SIZE * PAGE_SIZE; // 16MiB

// #[global_allocator]
// pub static ALLOCATOR: Allocator = Allocator {
//     start_virt_addr: UnsafeCell::new(0),
//     base_virt_addr: UnsafeCell::new(0),
// };

// pub struct Allocator {
//     start_virt_addr: UnsafeCell<u64>,
//     base_virt_addr: UnsafeCell<u64>,
// }

// unsafe impl Sync for Allocator {}

// unsafe impl GlobalAlloc for Allocator {
//     unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
//         let size = layout.size() as u64;
//         let align = layout.align() as u64;
//         let offset = (size + (align - 1)) & !(align - 1);
//         let start_virt_addr = self.start_virt_addr.get();
//         let base_virt_addr = self.base_virt_addr.get();
//         let base_virt_addr_clone = (*base_virt_addr).clone();

//         if *start_virt_addr == 0 && *base_virt_addr == 0 {
//             panic!("Heap was used before heap allocator was initialized");
//         }

//         if size > HEAP_SIZE as u64 {
//             return null_mut();
//         }

//         if align > size {
//             return null_mut();
//         }

//         if *base_virt_addr + offset > *start_virt_addr + HEAP_SIZE as u64 {
//             return null_mut();
//         }

//         *base_virt_addr += offset;
//         return base_virt_addr_clone as *mut u8;
//     }

//     unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
// }

// impl Allocator {
//     pub fn init(&self) {
//         // allocate for heap area
//         if let Ok(mem_frame_info) = BITMAP_MEM_MAN
//             .try_lock().unwrap()
//             .alloc_multi_mem_frame(HEAP_SIZE / UEFI_PAGE_SIZE)
//         {
//             let virt_addr = mem_frame_info.get_frame_start_virt_addr();
//             unsafe {
//                 (*self.start_virt_addr.get()) = virt_addr.get();
//                 (*self.base_virt_addr.get()) = virt_addr.get();
//             }

//             info!("mem: Initialized heap allocator");
//         } else {
//             panic!("mem: Failed to allocate heap area");
//         }
//     }
// }

// use linked_list_allocator crate
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init_heap() -> Result<()> {
    let mem_frame_info = bitmap::alloc_mem_frame((HEAP_SIZE / PAGE_SIZE).max(1))?;
    let frame_start_virt_addr = mem_frame_info.frame_start_virt_addr()?;
    bitmap::mem_clear(&mem_frame_info)?;

    unsafe {
        ALLOCATOR.try_lock().ok_or(MutexError::Locked)?.init(
            frame_start_virt_addr.as_ptr_mut(),
            mem_frame_info.frame_size,
        )
    }
    Ok(())
}

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("Allocation error: {:?}", layout);
}

#[test_case]
fn test_alloc_string() {
    let s1 = String::from("Hello, World!");
    assert_eq!(s1, "Hello, World!");
    let s2 = String::from("hoge huga hogera piyo 012345!\"#$%&");
    assert_eq!(s2, "hoge huga hogera piyo 012345!\"#$%&");
}

#[test_case]
fn test_alloc_long_string() {
    let len = 100000;
    let mut s = String::new();
    for _ in 0..len {
        s.push('a');
    }
    assert_eq!(s.len(), len);

    for c in s.chars() {
        assert_eq!(c, 'a');
    }
}

#[test_case]
fn test_alloc_vec() {
    let mut v = Vec::new();

    for i in 0..1000 {
        v.push(i as u64);
    }

    assert_eq!(v.len(), 1000);
    assert_eq!(v.capacity(), 1024);
    assert_eq!(v[0], 0);
    assert_eq!(v[1], 1);
    assert_eq!(v[2], 2);
    assert_eq!(v[997], 997);
    assert_eq!(v[998], 998);
    assert_eq!(v[999], 999);
    assert!(v.get(1000).is_none());
}
