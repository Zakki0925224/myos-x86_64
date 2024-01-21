use core::mem::size_of;

use alloc::vec::Vec;

const MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    Bit32,
    Bit64,
    Other(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Data {
    LittleEndian,
    BigEndian,
    Other(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Relocatable,
    Executable,
    Shared,
    Core,
    Other(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Machine {
    None,
    Sparc,
    X86,
    Mips,
    PowerPc,
    Arm,
    SuperH,
    Ia64,
    X8664,
    Aarch64,
    RiscV,
    Other(u16),
}

#[derive(Debug)]
#[repr(C)]
pub struct Elf64Header {
    pub magic: [u8; 4],
    class: u8,
    data: u8,
    pub version0: u8,
    pub os_abi: u8,
    pub abi_version: u8,
    reserved: [u8; 7],
    type_: u16,
    machine: u16,
    pub verison1: u8,
    pub entry_point: u64,
    pub ph_offset: u64,
    pub sh_offset: u64,
    pub flags: u32,
    pub eh_size: u16,
    pub ph_entry_size: u16,
    pub ph_num: u16,
    pub sh_entry_size: u16,
    pub sh_num: u16,
    pub sh_str_index: u16,
}

impl Elf64Header {
    pub fn is_valid(&self) -> bool {
        self.magic == MAGIC
    }

    pub fn class(&self) -> Class {
        match self.class {
            1 => Class::Bit32,
            2 => Class::Bit64,
            x => Class::Other(x),
        }
    }

    pub fn data(&self) -> Data {
        match self.data {
            1 => Data::LittleEndian,
            2 => Data::BigEndian,
            x => Data::Other(x),
        }
    }

    pub fn elf_type(&self) -> Type {
        match self.type_ {
            1 => Type::Relocatable,
            2 => Type::Executable,
            3 => Type::Shared,
            4 => Type::Core,
            x => Type::Other(x),
        }
    }

    pub fn machine(&self) -> Machine {
        match self.machine {
            0x00 => Machine::None,
            0x02 => Machine::Sparc,
            0x03 => Machine::X86,
            0x08 => Machine::Mips,
            0x14 => Machine::PowerPc,
            0x28 => Machine::Arm,
            0x2a => Machine::SuperH,
            0x32 => Machine::Ia64,
            0x3e => Machine::X8664,
            0xb7 => Machine::Aarch64,
            0xf3 => Machine::RiscV,
            x => Machine::Other(x),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentType {
    Null,
    Load,
    Dynamic,
    Interpreter,
    Note,
    Reserved,
    ProgramHeader,
    ThreadLocalStorage,
    OsSpecLow,
    OsSpecHigh,
    ProcSpecLow,
    ProcSpecHigh,
    Other(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentFlags {
    Executable,
    Writable,
    Readable,
    Other(u32),
}

#[derive(Debug)]
#[repr(C)]
pub struct Elf64ProgramHeader {
    segment_type: u32,
    flags: u32,
    pub offset: u64,
    pub vart_addr: u64,
    pub phys_addr: u64,
    pub file_size: u64,
    pub mem_size: u64,
    pub align: u64,
}

impl Elf64ProgramHeader {
    pub fn segment_type(&self) -> SegmentType {
        match self.segment_type {
            0x00000000 => SegmentType::Null,
            0x00000001 => SegmentType::Load,
            0x00000002 => SegmentType::Dynamic,
            0x00000003 => SegmentType::Interpreter,
            0x00000004 => SegmentType::Note,
            0x00000005 => SegmentType::Reserved,
            0x00000006 => SegmentType::ProgramHeader,
            0x00000007 => SegmentType::ThreadLocalStorage,
            0x60000000 => SegmentType::OsSpecLow,
            0x6fffffff => SegmentType::OsSpecHigh,
            0x70000000 => SegmentType::ProcSpecLow,
            0x7fffffff => SegmentType::ProcSpecHigh,
            x => SegmentType::Other(x),
        }
    }

    pub fn flags(&self) -> SegmentFlags {
        match self.flags {
            0x1 => SegmentFlags::Executable,
            0x2 => SegmentFlags::Writable,
            0x3 => SegmentFlags::Readable,
            x => SegmentFlags::Other(x),
        }
    }
}

#[derive(Debug)]
pub enum Elf64Error {
    InvalidMagicNumberError,
}

#[derive(Debug)]
pub struct Elf64<'a> {
    data: &'a [u8],
}

impl<'a> Elf64<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self, Elf64Error> {
        let header = unsafe { (data.as_ptr() as *const Elf64Header).read_volatile() };

        if !header.is_valid() {
            return Err(Elf64Error::InvalidMagicNumberError);
        }

        Ok(Self { data })
    }

    pub fn read_header(&self) -> Elf64Header {
        unsafe { (self.data.as_ptr() as *const Elf64Header).read_volatile() }
    }

    pub fn program_headers(&self) -> Vec<Elf64ProgramHeader> {
        let header = self.read_header();
        let ph_len = header.ph_num;
        let ph_offset = header.ph_offset;

        let mut phs = Vec::new();
        for i in 0..ph_len {
            let ph = unsafe {
                (self.data.as_ptr().offset(
                    ph_offset as isize + size_of::<Elf64ProgramHeader>() as isize * i as isize,
                ) as *const Elf64ProgramHeader)
                    .read_volatile()
            };
            phs.push(ph);
        }

        phs
    }
}
