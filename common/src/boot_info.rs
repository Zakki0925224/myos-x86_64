use crate::{graphic_info::GraphicInfo, mem_desc::MemoryDescriptor};

#[derive(Debug)]
#[repr(C)]
pub struct BootInfo<'a> {
    pub mem_map: &'a [MemoryDescriptor],
    pub graphic_info: GraphicInfo,
    pub initramfs_start_virt_addr: u64,
    pub initramfs_page_cnt: u64,
}

impl<'a> BootInfo<'a> {
    pub fn new(
        mem_map_slice: &'a [MemoryDescriptor],
        graphic_info: GraphicInfo,
        initramfs_start_virt_addr: u64,
        initramfs_page_cnt: u64,
    ) -> Self {
        Self {
            mem_map: mem_map_slice,
            graphic_info,
            initramfs_start_virt_addr,
            initramfs_page_cnt,
        }
    }
}
