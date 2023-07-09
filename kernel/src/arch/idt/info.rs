use bitflags::bitflags;

use crate::arch::addr::VirtualAddress;

// https://github.com/rust-osdev/x86_64/blob/master/src/structures/idt.rs
bitflags! {
    #[repr(transparent)]
    #[derive(Debug)]
    pub struct PageFaultErrorCode: u64 {
        const PROTECTION_VIOLATION = 1;
        const CAUSED_BY_WRITE = 1 << 1;
        const USER_MODE = 1 << 2;
        const MALFORMED_TABLE = 1 << 3;
        const INSTRUCTION_FETCH = 1 << 4;
        const PROTECTION_KEY = 1 << 5;
        const SHADOW_STACK = 1 << 6;
        const SGX = 1 << 15;
        const RMP = 1 << 31;
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
