use crate::{graphic_info::GraphicInfo, kernel_config::KernelConfig, mem_desc::MemoryDescriptor};

#[derive(Debug)]
#[repr(C)]
pub struct BootInfo<'a> {
    pub mem_map: &'a [MemoryDescriptor],
    pub graphic_info: GraphicInfo,
    pub initramfs_start_virt_addr: u64,
    pub initramfs_page_cnt: usize,
    pub rsdp_virt_addr: Option<u64>,
    pub kernel_config: KernelConfig<'a>,
}
