use alloc::{format, string::String};

use crate::arch::addr::VirtualAddress;

// https://github.com/rust-osdev/x86_64/blob/master/src/structures/idt.rs
#[repr(transparent)]
pub struct PageFaultErrorCode(u64);

impl PageFaultErrorCode {
    pub const PROTECTION_VIOLATION: u64 = 1;
    pub const CAUSED_BY_WRITE: u64 = 1 << 1;
    pub const USER_MODE: u64 = 1 << 2;
    pub const MALFORMED_TABLE: u64 = 1 << 3;
    pub const INSTRUCTION_FETCH: u64 = 1 << 4;
    pub const PROTECTION_KEY: u64 = 1 << 5;
    pub const SHADOW_STACK: u64 = 1 << 6;
    pub const SGX: u64 = 1 << 15;
    pub const RMP: u64 = 1 << 31;
}

impl core::fmt::Debug for PageFaultErrorCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut fmt = String::from("PageFaultErrorCode");

        fmt = format!("{}(0x{:x}) {{ ", fmt, self.0);

        if (self.0 & Self::PROTECTION_VIOLATION) != 0 {
            fmt = format!("{}PROTECTION_VIOLATION, ", fmt);
        }

        if (self.0 & Self::CAUSED_BY_WRITE) != 0 {
            fmt = format!("{}CAUSED_BY_WRITE, ", fmt);
        }

        if (self.0 & Self::USER_MODE) != 0 {
            fmt = format!("{}USER_MODE, ", fmt);
        }

        if (self.0 & Self::MALFORMED_TABLE) != 0 {
            fmt = format!("{}MALFORMED_TABLE, ", fmt);
        }

        if (self.0 & Self::INSTRUCTION_FETCH) != 0 {
            fmt = format!("{}INSTRUCTION_FETCH, ", fmt);
        }

        if (self.0 & Self::PROTECTION_KEY) != 0 {
            fmt = format!("{}PROTECTION_KEY, ", fmt);
        }

        if (self.0 & Self::SHADOW_STACK) != 0 {
            fmt = format!("{}SHADOW_STACK, ", fmt);
        }

        if (self.0 & Self::SGX) != 0 {
            fmt = format!("{}SGX, ", fmt);
        }

        if (self.0 & Self::RMP) != 0 {
            fmt = format!("{}RMP, ", fmt);
        }

        fmt = format!("{}}}", fmt);

        write!(f, "{}", fmt)
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct InterruptStackFrame {
    pub ins_ptr: VirtualAddress,
    pub code_seg: u64,
    pub cpu_flags: u64,
    pub stack_ptr: VirtualAddress,
    pub stack_seg: u64,
}
