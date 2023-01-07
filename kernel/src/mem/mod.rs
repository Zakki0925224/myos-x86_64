use common::mem_desc::MemoryDescriptor;

use crate::{arch::addr::VirtualAddress, mem::paging::PAGING, println};

use self::bitmap::BITMAP_MEM_MAN;

pub mod bitmap;
pub mod paging;

pub fn init(mem_map: &[MemoryDescriptor])
{
    BITMAP_MEM_MAN.lock().init(mem_map);
    let used = BITMAP_MEM_MAN.lock().get_used_mem_size();
    let total = BITMAP_MEM_MAN.lock().get_total_mem_size();
    println!("Memory used: {}B/{}B ({}%)", used, total, (used as f32 / total as f32) * 100f32);

    // let virt = VirtualAddress::new(0xdeadbeaf_dead);
    // println!("map to identity...");
    // PAGING.lock().map_to_identity(&virt);
    // println!("calc phys addr...");
    // println!("{:?}", PAGING.lock().calc_phys_addr(&virt).unwrap());

    // let virt = VirtualAddress::new(0x100000);
    // PAGING.lock().pml4_table_addr = BITMAP_MEM_MAN
    //     .lock()
    //     .alloc_single_mem_frame()
    //     .unwrap()
    //     .get_frame_start_virt_addr()
    //     .get_phys_addr();
    // PAGING.lock().map_to_identity(&virt).unwrap();
    // println!("0x{:x}", virt.get_phys_addr().get());
    PAGING.lock().create_new_page_table();
}
