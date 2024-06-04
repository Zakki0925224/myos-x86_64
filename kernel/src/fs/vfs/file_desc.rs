use crate::error::{Error, Result};

use super::FileId;
use core::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileDescriptorNumber(u64);

impl FileDescriptorNumber {
    pub const STDIN: Self = Self(0);
    pub const STDOUT: Self = Self(1);
    pub const STDERR: Self = Self(2);

    pub fn new() -> Self {
        static NEXT_NUM: AtomicU64 = AtomicU64::new(3);
        Self(NEXT_NUM.fetch_add(1, Ordering::Relaxed))
    }

    pub const fn new_val(value: i64) -> Result<Self> {
        if value < 0 {
            return Err(Error::Failed("Invalid file descriptor number"));
        }

        Ok(Self(value as u64))
    }

    pub fn get(&self) -> u64 {
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
