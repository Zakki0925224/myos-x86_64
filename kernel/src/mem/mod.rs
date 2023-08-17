use common::mem_desc::MemoryDescriptor;

use crate::{
    arch::addr::{Address, VirtualAddress},
    mem::allocator::ALLOCATOR,
    mem::paging::PAGE_MAN,
    println,
};

use self::bitmap::BITMAP_MEM_MAN;

pub mod allocator;
pub mod bitmap;
pub mod paging;

pub fn init(mem_map: &[MemoryDescriptor]) {
    if let Err(err) = BITMAP_MEM_MAN.lock().init(mem_map) {
        panic!("mem: {:?}", err);
    }

    ALLOCATOR.init();

    // TODO: not working
    // match PAGE_MAN.lock().create_new_page_table() {
    //     Ok(_) => (),
    //     Err(err) => println!("{:?}", err),
    // }
    println!(
        "{:?}",
        PAGE_MAN.lock().calc_phys_addr(VirtualAddress::new(0x1000))
    );

    let used = BITMAP_MEM_MAN.lock().get_used_mem_size();
    let total = BITMAP_MEM_MAN.lock().get_total_mem_size();
    println!(
        "Memory used: {}B/{}B ({}%)",
        used,
        total,
        (used as f32 / total as f32) * 100f32
    );
}
