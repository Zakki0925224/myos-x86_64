use common::mem_desc::MemoryDescriptor;

use crate::{
    arch::addr::VirtualAddress, error::Result, mem::paging::PAGE_MAN, println,
    util::mutex::MutexError,
};

use self::bitmap::BITMAP_MEM_MAN;

pub mod allocator;
pub mod bitmap;
pub mod buffer;
pub mod paging;

pub fn init(mem_map: &[MemoryDescriptor]) {
    if let Err(err) = BITMAP_MEM_MAN.try_lock().unwrap().init(mem_map) {
        panic!("mem: {:?}", err);
    }

    allocator::init_heap();

    // TODO: not working
    // match PAGE_MAN.try_lock().unwrap().create_new_page_table() {
    //     Ok(_) => (),
    //     Err(err) => println!("{:?}", err),
    // }
    assert!(
        PAGE_MAN
            .try_lock()
            .unwrap()
            .calc_phys_addr(VirtualAddress::new(0x1000))
            .unwrap()
            .get()
            == 0x1000
    );
}

pub fn free() -> Result<()> {
    if let Ok(mem_man) = BITMAP_MEM_MAN.try_lock() {
        let used = mem_man.get_used_mem_size();
        let total = mem_man.get_total_mem_size();

        println!(
            "Memory used: {}B/{}B ({}%)",
            used,
            total,
            (used as f32 / total as f32) * 100f32
        );

        return Ok(());
    } else {
        return Err(MutexError::Locked.into());
    }
}
