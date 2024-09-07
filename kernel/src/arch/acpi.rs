use core::{mem::size_of, ptr::read_unaligned, slice};

use crate::{
    error::Result,
    mem::paging::{self, EntryMode, PageWriteThroughLevel, ReadWrite, PAGE_SIZE},
    println,
};

use super::addr::VirtualAddress;

static mut ACPI: Acpi = Acpi::new();

const RSDP_SIGNATURE: [u8; 8] = *b"RSD PTR ";
const XSDT_SIGNATURE: [u8; 4] = *b"XSDT";
const FADT_SIGNATURE: [u8; 4] = *b"FACP";

#[derive(Debug)]
#[repr(C, packed)]
struct RootSystemDescriptorPointer {
    sign: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    rev: u8,
    rsdt_addr: u32,
    len: u32,
    xsdt_addr: u64,
    ext_checksum: u8,
    reserved: [u8; 3],
}

impl RootSystemDescriptorPointer {
    fn is_valid(&self) -> bool {
        self.sign == RSDP_SIGNATURE
    }

    fn is_valid_checksum(&self) -> bool {
        let size = size_of::<Self>();

        let mut sum: u8 = 0;
        for i in 0..size {
            let byte = unsafe { read_unaligned((self as *const _ as *const u8).add(i)) };
            sum = sum.wrapping_add(byte);
        }

        sum == 0
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct DescriptionHeader {
    sign: [u8; 4],
    len: u32,
    rev: u8,
    checksum: u8,
    oem_id: [u8; 6],
    oem_table_id: [u8; 8],
    oem_rev: u32,
    creator_id: u32,
    creator_rev: u32,
}

impl DescriptionHeader {
    fn is_valid(&self, sign: [u8; 4]) -> bool {
        self.sign == sign
    }

    fn is_valid_checksum(&self) -> bool {
        let bytes: &[u8] =
            unsafe { slice::from_raw_parts(self as *const _ as *const u8, self.len as usize) };

        bytes.iter().fold(0u8, |acc, &b| acc.wrapping_add(b)) == 0
    }

    fn entries_count(&self) -> usize {
        (self.len as usize - size_of::<Self>()) / size_of::<u64>()
    }
}

#[derive(Debug)]
#[repr(C, packed)]
struct FixedAcpiDescriptionTable {
    header: DescriptionHeader,
    reserved0: [u8; 36],
    pm_timer_block: u32,
    reserved1: [u8; 32],
    flags: u32,
    reserved2: [u8; 148],
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AcpiError {
    InvalidSignatureError,
    InvalidRevisionError(u8),
    InvalidChecksumError,
    NotInitialized,
}

struct Acpi {
    rsdp_virt_addr: Option<VirtualAddress>,
}

impl Acpi {
    const fn new() -> Self {
        Self {
            rsdp_virt_addr: None,
        }
    }

    fn init(&mut self, rsdp_virt_addr: VirtualAddress) -> Result<()> {
        let rsdp = unsafe { &*(rsdp_virt_addr.as_ptr() as *const RootSystemDescriptorPointer) };
        let rev = rsdp.rev;

        if !rsdp.is_valid() {
            return Err(AcpiError::InvalidSignatureError.into());
        }

        if rev != 2 {
            return Err(AcpiError::InvalidRevisionError(rev).into());
        }

        if !rsdp.is_valid_checksum() {
            return Err(AcpiError::InvalidChecksumError.into());
        }

        self.rsdp_virt_addr = Some(rsdp_virt_addr);
        Ok(())
    }

    fn rsdp(&self) -> Result<&RootSystemDescriptorPointer> {
        self.rsdp_virt_addr
            .map(|addr| unsafe { &*(addr.as_ptr() as *const RootSystemDescriptorPointer) })
            .ok_or(AcpiError::NotInitialized.into())
    }

    // XSDT header, entries
    fn xsdt(&self) -> Result<(&DescriptionHeader, &[u64])> {
        let rsdp = self.rsdp()?;
        let xsdt_virt_addr: VirtualAddress = rsdp.xsdt_addr.into();
        let xsdt = unsafe { &*(xsdt_virt_addr.as_ptr() as *const DescriptionHeader) };

        if !xsdt.is_valid(XSDT_SIGNATURE) {
            return Err(AcpiError::InvalidSignatureError.into());
        }

        if !xsdt.is_valid_checksum() {
            return Err(AcpiError::InvalidChecksumError.into());
        }

        let entries = unsafe {
            slice::from_raw_parts(
                xsdt_virt_addr
                    .offset(size_of::<DescriptionHeader>() + 4)
                    .as_ptr(),
                xsdt.entries_count(),
            )
        };

        Ok((xsdt, entries))
    }

    // TODO: #GP fault
    fn fadt(&self) -> Result<Option<&FixedAcpiDescriptionTable>> {
        let (_, xsdt_entries) = self.xsdt()?;
        let mut fadt = None;

        println!("{:?}", xsdt_entries);
        for entry_addr in xsdt_entries {
            let entry_addr: VirtualAddress = (*entry_addr).into();

            if entry_addr.get() == 0 {
                continue;
            }

            paging::update_mapping(
                entry_addr,
                entry_addr.offset(PAGE_SIZE),
                entry_addr.get().into(),
                ReadWrite::Read,
                EntryMode::Supervisor,
                PageWriteThroughLevel::WriteThrough,
            )?;

            let entry = unsafe { &*(entry_addr.as_ptr() as *const FixedAcpiDescriptionTable) };
            if entry.header.is_valid(FADT_SIGNATURE) {
                fadt = Some(entry);
                break;
            }
        }

        Ok(fadt)
    }
}

pub fn init(rsdp_virt_addr: VirtualAddress) -> Result<()> {
    unsafe { ACPI.init(rsdp_virt_addr) }?;
    let fadt = unsafe { ACPI.fadt() }?.ok_or(AcpiError::NotInitialized)?;
    println!("{:?}", fadt);

    Ok(())
}
