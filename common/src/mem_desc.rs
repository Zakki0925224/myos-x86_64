#[derive(Debug, Copy, Clone)]
pub enum MemoryType
{
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

#[derive(Debug, Copy, Clone)]
#[repr(u64)]
pub enum MemoryAttribute
{
    Uncacheable = 0x1,
    WriteCombine = 0x2,
    WriteThrough = 0x4,
    WriteBack = 0x8,
    UncachableExported = 0x10,
    WriteProtect = 0x1000,
    ReadProtect = 0x2000,
    ExecuteProtect = 0x4000,
    NonVolatile = 0x8000,
    MoreReliable = 0x10000,
    ReadOnly = 0x20000,
    Runtime = 0x8000_0000_0000_0000,
}

#[derive(Debug, Copy, Clone)]
pub struct MemoryDescriptor
{
    pub ty: MemoryType,
    pub phys_start: u64,
    pub virt_start: u64,
    pub page_cnt: u64,
    pub attr: MemoryAttribute,
}
