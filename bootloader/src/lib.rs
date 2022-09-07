#![no_std]

use uefi::proto::console::gop::ModeInfo;

pub mod config;

#[derive(Debug)]
#[repr(C)]
pub struct BootInfo
{
    // mem_map,
    pub graphic_info: GraphicInfo
    // phys_mem_offset,
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
    pub framebuf_size: u64
}