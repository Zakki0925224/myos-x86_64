use core::slice;

use crate::{graphic_info::GraphicInfo, mem_desc::MemoryDescriptor};

#[derive(Debug)]
#[repr(C)]
pub struct BootInfo
{
    mem_map: *const MemoryDescriptor,
    mem_map_len: u64,
    pub graphic_info: GraphicInfo, // phys_mem_offset,
                                   // cmdline,
                                   // initramfs_addr,
                                   // initramfs_size
}

impl BootInfo
{
    pub fn new(
        mem_map_slice: &[MemoryDescriptor],
        mem_map_len: usize,
        graphic_info: GraphicInfo,
    ) -> Self
    {
        return Self {
            mem_map: mem_map_slice.as_ptr() as *const MemoryDescriptor,
            mem_map_len: mem_map_len as u64,
            graphic_info,
        };
    }

    pub fn get_mem_map(&self) -> &[MemoryDescriptor]
    {
        unsafe {
            return slice::from_raw_parts(self.mem_map, self.mem_map_len as usize);
        }
    }
}
