use crate::{
    arch::addr::VirtualAddress,
    mem::paging::{EntryMode, PageWriteThroughLevel, ReadWrite, PAGE_SIZE},
    println,
};
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

    let (_, max) = bitmap::get_mem_size().unwrap();
    let start = PAGE_SIZE as u64;
    let end = max as u64;
    if let Err(err) = paging::create_new_page_table(
        start.into(),
        end.into(),
        start.into(),
        ReadWrite::Write,
        EntryMode::Supervisor,
        PageWriteThroughLevel::WriteThrough,
    ) {
        error!("paging: Failed to create new page table: {:?}", err);
    }
    info!(
        "paging: Created new page table (mapped identity 0x{:x} to 0x{:x})",
        start, end
    );

    if let Err(err) = allocator::init_heap() {
        panic!("mem: {:?}", err);
    }
    info!("mem: Initialized heap allocator");

    assert_eq!(
        paging::calc_phys_addr(VirtualAddress::new(0xabcd000))
            .unwrap()
            .get(),
        0xabcd000
    );
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
