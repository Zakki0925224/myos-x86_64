use alloc::vec::Vec;
use uefi::{proto::console::gop::ModeInfo, table::boot::MemoryDescriptor};

#[derive(Debug)]
#[repr(C)]
pub struct BootInfo
{
    pub mem_map: Vec<&'static MemoryDescriptor>,
    pub graphic_info: GraphicInfo, // phys_mem_offset,
                                   // cmdline,
                                   // initramfs_addr,
                                   // initramfs_size
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct GraphicInfo
{
    pub mode: ModeInfo,
    pub framebuf_addr: u64,
    pub framebuf_size: u64,
}
