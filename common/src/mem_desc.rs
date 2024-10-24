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
    Other(u32),
}

impl MemoryType {
    pub fn is_available_memory(&self) -> bool {
        *self == Self::BootServicesCode
            || *self == Self::BootServicesData
            || *self == Self::Conventional
    }
}

#[derive(Debug, Copy, Clone)]
pub struct MemoryDescriptor {
    pub ty: MemoryType,
    pub phys_start: u64,
    pub virt_start: u64,
    pub page_cnt: u64,
    pub attr: u64,
}
