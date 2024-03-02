use crate::{arch::addr::VirtualAddress, println};
use common::mem_desc::MemoryDescriptor;
use log::{error, info};

pub mod allocator;
pub mod bitmap;
pub mod paging;

pub fn init(mem_map: &[MemoryDescriptor]) {
    if let Err(err) = bitmap::init(mem_map) {
        panic!("mem: {:?}", err);
    }
    info!("mem: Initialized bitmap memory manager");

    // TODO: not working
    // match paging::create_new_page_table() {
    //     Ok(_) => (),
    //     Err(err) => error!("mem: {:?}", err),
    // }

    assert_eq!(
        paging::calc_phys_addr(VirtualAddress::new(0xabcd000))
            .unwrap()
            .get(),
        0xabcd000
    );

    allocator::init_heap();
    info!("mem: Initialized heap allocator");
}

pub fn free() {
    let (used, total) = bitmap::get_mem_size().unwrap_or((0, 0));

    println!(
        "Memory used: {}B/{}B ({}%)",
        used,
        total,
        (used as f32 / total as f32) * 100f32
    );
}
