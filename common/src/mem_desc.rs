use bitflags::bitflags;

pub const UEFI_PAGE_SIZE: usize = 0x1000;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MemoryType {
    Reserved,
    LoaderCode,
    LoaderData,
    BootServicesCode,
    BootServicesData,
    RuntimeServicesCode,
    RuntimeServicesData,
    Conventional,
    Unusable,
    AcpiReclaim,
    AcpiNonVolatile,
    Mmio,
    MmioPortSpace,
    PalCode,
    PersistentMemory,
    Custom(u32),
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct MemoryAttribute: u64
    {
        const UNCACHEABLE = 0x1;
        const WRITE_COMBINE = 0x2;
        const WRITE_THROUGH = 0x4;
        const WRITE_BACK = 0x8;
        const UNCACHABLE_EXPORTED = 0x10;
        const WRITE_PROTECT = 0x1000;
        const READ_PROTECT = 0x2000;
        const EXECUTE_PROTECT = 0x4000;
        const NON_VOLATILE = 0x8000;
        const MORE_RELIABLE = 0x10000;
        const READ_ONLY = 0x20000;
        const RUNTIME = 0x8000_0000_0000_0000;
    }
}

#[derive(Debug, Copy, Clone)]
pub struct MemoryDescriptor {
    pub ty: MemoryType,
    pub phys_start: u64,
    pub virt_start: u64,
    pub page_cnt: u64,
    pub attr: MemoryAttribute,
}
