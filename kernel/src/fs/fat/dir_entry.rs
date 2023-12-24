use alloc::string::String;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Attribute {
    ReadOnly = 0x01,
    Hidden = 0x02,
    System = 0x04,
    VolumeLabel = 0x08,
    LongFileName = 0x0f,
    Directory = 0x10,
    Archive = 0x20,
    Device = 0x40,
}

#[derive(Debug, PartialEq, Eq)]
pub enum EntryType {
    Null,
    Unused,
    LongFileName,
    Data,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct DirectoryEntry {
    name: [u8; 11],
    attr: u8,
    nt_reserved: u8,
    create_time_ms: u8,
    create_time: [u8; 2],
    create_date: [u8; 2],
    last_access_date: [u8; 2],
    first_cluster_high: u16,
    wrote_time: [u8; 2],
    wrote_date: [u8; 2],
    first_cluster_low: u16,
    file_size: [u8; 4],
}

impl DirectoryEntry {
    pub fn name(&self) -> String {
        String::from_utf8_lossy(&self.name).into_owned()
    }

    pub fn attr(&self) -> Option<Attribute> {
        match self.attr {
            0x01 => Some(Attribute::ReadOnly),
            0x02 => Some(Attribute::Hidden),
            0x04 => Some(Attribute::System),
            0x08 => Some(Attribute::VolumeLabel),
            0x0f => Some(Attribute::LongFileName),
            0x10 => Some(Attribute::Directory),
            0x20 => Some(Attribute::Archive),
            0x40 => Some(Attribute::Device),
            _ => None,
        }
    }

    pub fn entry_type(&self) -> EntryType {
        match self.name[0] {
            0x00 => return EntryType::Null,
            0xe5 => return EntryType::Unused,
            _ => (),
        }

        match self.name[10] {
            0x0f => EntryType::LongFileName,
            _ => EntryType::Data,
        }
    }

    pub fn first_cluster_num(&self) -> usize {
        ((self.first_cluster_high as u32) << 16 | self.first_cluster_low as u32) as usize
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct LongFileNameEntry {
    sequence_num: u8,
    file_name_0: [u8; 10],
    attr: u8,
    long_file_name_type: u8,
    checksum: u8,
    file_name_1: [u8; 12],
    first_cluster_num: u16,
    file_name_2: [u8; 4],
}
