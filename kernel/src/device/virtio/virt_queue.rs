use crate::{
    addr::VirtualAddress,
    error::{Error, Result},
    mem::{
        bitmap::{self, MemoryFrameInfo},
        paging::PAGE_SIZE,
    },
};
use core::mem::size_of;

#[derive(Debug, Default)]
#[repr(C, packed)]
pub struct QueueDescriptor {
    addr: u64,
    len: u32,
    flags: u16,
    next: u16,
}

#[repr(C)]
pub struct QueueAvailableHeader {
    flags: u16,
    index: u16,
}

#[repr(C)]
pub struct QueueUsedHeader {
    flags: u16,
    index: u16,
}

#[repr(C)]
pub struct QueueUsedElement {
    id: u32,
    len: u32,
}

pub struct Queue {
    mem_frame_info: MemoryFrameInfo,
    base_virt_addr: VirtualAddress,
    queue_size: usize,
}

impl Drop for Queue {
    fn drop(&mut self) {
        bitmap::dealloc_mem_frame(self.mem_frame_info).unwrap();
    }
}

impl Queue {
    pub fn init(mem_frame_info: MemoryFrameInfo, queue_size: usize) -> Result<Self> {
        if mem_frame_info.frame_start_phys_addr.get() & 0x0fff != 0 {
            return Err(Error::Failed("Physical address not aligned by 4K"));
        }

        // clear memory
        bitmap::mem_clear(&mem_frame_info)?;

        Ok(Self {
            mem_frame_info,
            base_virt_addr: mem_frame_info.frame_start_virt_addr()?,
            queue_size,
        })
    }

    pub fn read_desc(&self, index: usize) -> Result<QueueDescriptor> {
        if index >= self.queue_size {
            return Err(Error::IndexOutOfBoundsError(index));
        }

        let desc = unsafe {
            self.base_virt_addr
                .offset(self.offset_of_descs() + size_of::<QueueDescriptor>() * index)
                .as_ptr::<QueueDescriptor>()
                .read()
        };

        Ok(desc)
    }

    pub fn write_desc(&self, index: usize, desc: QueueDescriptor) -> Result<()> {
        if index >= self.queue_size {
            return Err(Error::IndexOutOfBoundsError(index));
        }

        unsafe {
            self.base_virt_addr
                .offset(self.offset_of_descs() + size_of::<QueueDescriptor>() * index)
                .as_ptr_mut::<QueueDescriptor>()
                .write(desc);
        }

        Ok(())
    }

    fn bytes_of_descs(&self) -> usize {
        size_of::<QueueDescriptor>() * self.queue_size
    }

    fn bytes_of_queue_available(&self) -> usize {
        size_of::<QueueAvailableHeader>() + size_of::<u16>() * self.queue_size
    }

    fn bytes_of_queue_used(&self) -> usize {
        size_of::<QueueUsedHeader>() + size_of::<QueueUsedElement>() * self.queue_size
    }

    fn offset_of_descs(&self) -> usize {
        0
    }

    fn offset_of_queue_available(&self) -> usize {
        self.bytes_of_descs()
    }

    // next page
    fn offset_of_queue_used(&self) -> usize {
        ((self.bytes_of_descs() + self.bytes_of_queue_available()) / PAGE_SIZE + 1) * PAGE_SIZE
    }
}
