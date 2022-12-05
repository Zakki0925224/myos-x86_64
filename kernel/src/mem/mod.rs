use common::mem_desc::MemoryDescriptor;

use crate::{mem::paging::PAGING, println};

use self::bitmap::BITMAP_MEM_MAN;

pub mod bitmap;
pub mod paging;

pub fn init(mem_map: &[MemoryDescriptor])
{
    BITMAP_MEM_MAN.lock().init(mem_map);
    let used = BITMAP_MEM_MAN.lock().get_used_mem_size();
    let total = BITMAP_MEM_MAN.lock().get_total_mem_size();
    println!("Memory used: {}B/{}B ({}%)", used, total, (used as f32 / total as f32) * 100f32);

    PAGING.lock().test();
}
