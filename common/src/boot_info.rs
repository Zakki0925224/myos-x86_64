use crate::{graphic_info::GraphicInfo, kernel_config::KernelConfig, mem_desc::MemoryDescriptor};

#[derive(Debug)]
#[repr(C)]
pub struct BootInfo<'a> {
    pub mem_map: &'a [MemoryDescriptor],
    pub graphic_info: GraphicInfo,
    pub initramfs_start_virt_addr: u64,
    pub initramfs_page_cnt: u64,
    pub kernel_config: KernelConfig<'a>,
}

impl<'a> BootInfo<'a> {
    pub fn new(
        mem_map_slice: &'a [MemoryDescriptor],
        graphic_info: GraphicInfo,
        initramfs_start_virt_addr: u64,
        initramfs_page_cnt: u64,
        kernel_config: KernelConfig<'a>,
    ) -> Self {
        Self {
            mem_map: mem_map_slice,
            graphic_info,
            initramfs_start_virt_addr,
            initramfs_page_cnt,
            kernel_config,
        }
    }
}
