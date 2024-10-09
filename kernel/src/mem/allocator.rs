use super::{bitmap, paging::PAGE_SIZE};
use crate::error::Result;
use core::alloc::Layout;
use linked_list_allocator::LockedHeap;

const HEAP_SIZE: usize = PAGE_SIZE * PAGE_SIZE; // 16MiB

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init_heap() -> Result<()> {
    let mem_frame_info = bitmap::alloc_mem_frame((HEAP_SIZE / PAGE_SIZE).max(1))?;
    let frame_start_virt_addr = mem_frame_info.frame_start_virt_addr()?;
    bitmap::mem_clear(&mem_frame_info)?;

    unsafe {
        ALLOCATOR.lock().init(
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
    use alloc::string::String;

    let s1 = String::from("Hello, World!");
    assert_eq!(s1, "Hello, World!");
    let s2 = String::from("hoge huga hogera piyo 012345!\"#$%&");
    assert_eq!(s2, "hoge huga hogera piyo 012345!\"#$%&");
}

#[test_case]
fn test_alloc_long_string() {
    use alloc::string::String;

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
    use alloc::vec::Vec;

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
