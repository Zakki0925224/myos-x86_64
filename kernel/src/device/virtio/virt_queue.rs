use crate::{
    addr::{PhysicalAddress, VirtualAddress},
    error::{Error, Result},
    mem::{
        bitmap::{self, MemoryFrameInfo},
        paging::PAGE_SIZE,
    },
};
use core::{mem::size_of, slice::from_raw_parts_mut};

#[derive(Debug, Default)]
#[repr(C, packed)]
pub struct QueueDescriptor {
    pub addr: u64,
    pub len: u32,
    pub flags: u16,
    pub next: u16,
}

impl QueueDescriptor {
    pub fn data<T>(&self) -> &T {
        let ptr = self.virt_addr().as_ptr();
        unsafe { &*ptr }
    }

    pub fn write_data<T>(&self, data: &T) {
        self.virt_addr()
            .copy_from_nonoverlapping(data as *const T, 1);
    }

    fn virt_addr(&self) -> VirtualAddress {
        self.addr.into()
    }
}

#[repr(C)]
pub struct QueueAvailableHeader {
    pub flags: u16,
    pub index: u16,
}

#[repr(C)]
pub struct QueueUsedHeader {
    pub flags: u16,
    pub index: u16,
}

#[repr(C)]
pub struct QueueUsedElement {
    pub id: u32,
    pub len: u32,
}

#[derive(Debug)]
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
    pub fn new(queue_size: usize) -> Result<Self> {
        let bytes_of_descs = size_of::<QueueDescriptor>() * queue_size;
        let bytes_of_queue_available =
            size_of::<QueueAvailableHeader>() + size_of::<u16>() * queue_size;
        let bytes_of_queue_used =
            size_of::<QueueUsedHeader>() + size_of::<QueueUsedElement>() * queue_size;
        let queue_page_cnt = ((bytes_of_descs + bytes_of_queue_available) / PAGE_SIZE).max(1)
            + (bytes_of_queue_used / PAGE_SIZE).max(1);

        let mem_frame_info = bitmap::alloc_mem_frame(queue_page_cnt)?;

        if mem_frame_info.frame_start_phys_addr.get() % PAGE_SIZE as u64 != 0 {
            bitmap::dealloc_mem_frame(mem_frame_info)?;
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

    pub fn set_data<T>(&mut self, data: &T) -> Result<()> {
        let index = self.available_header_mut().index as usize % self.queue_size;
        let desc = &mut self.descs_mut()[index];

        if size_of::<T>() > desc.len as usize {
            return Err(Error::Failed("Data too long"));
        }

        desc.write_data(data);
        self.available_elements_mut()[index] = index as u16;
        self.available_header_mut().index = (index + 1) as u16;
        Ok(())
    }

    pub fn descs_mut(&self) -> &mut [QueueDescriptor] {
        let ptr_mut: *mut QueueDescriptor = self
            .base_virt_addr
            .offset(self.offset_of_descs())
            .as_ptr_mut();

        unsafe { from_raw_parts_mut(ptr_mut, self.queue_size) }
    }

    pub fn available_header_mut(&self) -> &mut QueueAvailableHeader {
        let ptr_mut: *mut QueueAvailableHeader = self
            .base_virt_addr
            .offset(self.offset_of_queue_available())
            .as_ptr_mut();

        unsafe { &mut *ptr_mut }
    }

    pub fn available_elements_mut(&self) -> &mut [u16] {
        let ptr_mut: *mut u16 = self
            .base_virt_addr
            .offset(self.offset_of_queue_available() + size_of::<QueueAvailableHeader>())
            .as_ptr_mut();

        unsafe { from_raw_parts_mut(ptr_mut, self.queue_size) }
    }

    pub fn used_header_mut(&self) -> &mut QueueUsedHeader {
        let ptr_mut: *mut QueueUsedHeader = self
            .base_virt_addr
            .offset(self.offset_of_queue_used())
            .as_ptr_mut();

        unsafe { &mut *ptr_mut }
    }

    pub fn used_elements_mut(&self) -> &mut [QueueUsedElement] {
        let ptr_mut: *mut QueueUsedElement = self
            .base_virt_addr
            .offset(self.offset_of_queue_used() + size_of::<QueueUsedHeader>())
            .as_ptr_mut();

        unsafe { from_raw_parts_mut(ptr_mut, self.queue_size) }
    }

    pub fn queue_size(&self) -> usize {
        self.queue_size
    }

    pub fn phys_addr(&self) -> &PhysicalAddress {
        &self.mem_frame_info.frame_start_phys_addr
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
