use core::sync::atomic::{AtomicU16, Ordering};

use alloc::string::String;

use super::FileId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileDescriptorNumber(u16);

impl FileDescriptorNumber {
    pub const STDIN: Self = Self(0);
    pub const STDOUT: Self = Self(1);
    pub const STDERR: Self = Self(2);

    pub fn new() -> Self {
        static NEXT_NUM: AtomicU16 = AtomicU16::new(3);
        Self(NEXT_NUM.fetch_add(1, Ordering::Relaxed))
    }

    pub const fn new_val(value: u16) -> Self {
        Self(value)
    }

    pub fn get(&self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Open,
    Close,
}

#[derive(Debug, Clone)]
pub struct FileDescriptor {
    pub num: FileDescriptorNumber,
    pub status: Status,
    pub file_id: FileId,
}
