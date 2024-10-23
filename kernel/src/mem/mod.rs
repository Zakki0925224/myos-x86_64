use crate::{
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
    info!("mem: Bitmap memory manager initialized");

    let (_, max) = bitmap::get_mem_size().unwrap();
    let start = PAGE_SIZE as u64;
    let end = max as u64;

    if let Err(err) = paging::create_new_page_table(
        start.into(),
        end.into(),
        start.into(),
        ReadWrite::Write,
        EntryMode::Supervisor,
        PageWriteThroughLevel::WriteBack,
    ) {
        error!("paging: Failed to create new page table: {:?}", err);
    }

    if let Err(err) = allocator::init_heap() {
        panic!("mem: {:?}", err);
    }
    info!("mem: Heap allocator initialized");
}

pub fn free() {
    fn format_size(size: usize) -> (f64, &'static str) {
        const KIB: usize = 1024;
        const MIB: usize = 1024 * KIB;
        const GIB: usize = 1024 * MIB;

        if size >= GIB {
            (size as f64 / GIB as f64, "GiB")
        } else if size >= MIB {
            (size as f64 / MIB as f64, "MiB")
        } else if size >= KIB {
            (size as f64 / KIB as f64, "KiB")
        } else {
            (size as f64, "B")
        }
    }

    let (used, total) = bitmap::get_mem_size().unwrap_or((0, 0));
    let (used_value, used_unit) = format_size(used);
    let (total_value, total_unit) = format_size(total);

    println!(
        "Memory used: {:.2}{}({}B) / {:.2}{}({}B) ({:.2}%)",
        used_value,
        used_unit,
        used,
        total_value,
        total_unit,
        total,
        (used as f64 / total as f64) * 100f64
    );
}
