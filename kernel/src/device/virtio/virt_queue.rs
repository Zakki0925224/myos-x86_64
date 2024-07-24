#[derive(Debug, Default)]
#[repr(C, packed)]
pub struct QueueDescriptor {
    addr: u64,
    len: u32,
    flags: u16,
    next: u16,
}

#[repr(C)]
pub struct QueueUsedElement {
    id: u32,
    len: u32,
}
