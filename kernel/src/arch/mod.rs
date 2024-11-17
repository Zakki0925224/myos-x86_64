use crate::error::{Error, Result};
use core::arch::asm;
use register::{
    control::{Cr0, Cr4},
    Register,
};

pub mod acpi;
pub mod addr;
pub mod apic;
pub mod context;
pub mod cpu;
pub mod gdt;
pub mod idt;
pub mod qemu;
pub mod register;
pub mod syscall;
pub mod task;
pub mod tss;

#[repr(C, packed(2))]
#[derive(Debug, Default)]
pub struct DescriptorTableArgs {
    pub limit: u16,
    pub base: u64,
}

pub fn enable_sse() -> Result<()> {
    let cpu_vi = cpu::version_info();

    if !cpu_vi.feature_sse {
        return Err(Error::Failed("CPU does not support SSE"));
    }

    if !cpu_vi.feature_sse2 {
        return Err(Error::Failed("CPU does not support SSE2"));
    }

    if !cpu_vi.feature_sse3 {
        return Err(Error::Failed("CPU does not support SSE3"));
    }

    if !cpu_vi.feature_ssse3 {
        return Err(Error::Failed("CPU does not support SSSE3"));
    }

    if !cpu_vi.feature_sse4_1 {
        return Err(Error::Failed("CPU does not support SSE4.1"));
    }

    if !cpu_vi.feature_sse4_2 {
        return Err(Error::Failed("CPU does not support SSE4.2"));
    }

    let mut cr0 = Cr0::read();
    cr0.set_emulation(false);
    cr0.set_monitor_coprocessor(true);
    cr0.write();
    cr0 = Cr0::read();
    assert_eq!(cr0.emulation(), false);
    assert_eq!(cr0.monitor_coprocessor(), true);

    let mut cr4 = Cr4::read();
    cr4.set_osfxsr(true);
    cr4.set_osxmmexcept(true);
    cr4.write();
    cr4 = Cr4::read();
    assert_eq!(cr4.osfxsr(), true);
    assert_eq!(cr4.osxmmexcept(), true);

    Ok(())
}

pub fn hlt() {
    unsafe { asm!("hlt") }
}

pub fn disabled_int<F: FnMut() -> R, R>(mut func: F) -> R {
    unsafe { asm!("cli") };
    let func_res = func();
    unsafe { asm!("sti") };
    func_res
}

pub fn int3() {
    unsafe { asm!("int3") }
}

pub fn out8(port: u16, data: u8) {
    unsafe {
        asm!(
            "out dx, al",
            in("dx") port,
            in("al") data
        );
    }
}

pub fn in8(port: u16) -> u8 {
    let data: u8;
    unsafe {
        asm!(
            "in al, dx",
            out("al") data,
            in("dx") port
        );
    }
    data
}

pub fn out16(port: u16, data: u16) {
    unsafe {
        asm!(
            "out dx, ax",
            in("dx") port,
            in("ax") data
        );
    }
}

pub fn in16(port: u16) -> u16 {
    let data: u16;
    unsafe {
        asm!(
            "in ax, dx",
            out("ax") data,
            in("dx") port
        );
    }
    data
}

pub fn out32(port: u32, data: u32) {
    unsafe {
        asm!(
            "out dx, eax",
            in("edx") port,
            in("eax") data
        );
    }
}

pub fn in32(port: u32) -> u32 {
    let data: u32;
    unsafe {
        asm!(
            "in eax, dx",
            out("eax") data,
            in("edx") port
        );
    }
    data
}

pub fn lidt(desc_table_args: &DescriptorTableArgs) {
    unsafe {
        asm!("lidt [{}]", in(reg) desc_table_args);
    }
}

pub fn lgdt(desc_table_args: &DescriptorTableArgs) {
    unsafe {
        asm!("lgdt [{}]", in(reg) desc_table_args);
    }
}

pub fn ltr(sel: u16) {
    unsafe {
        asm!("ltr cx", in("cx") sel);
    }
}

pub fn read_msr(addr: u32) -> u64 {
    let low: u32;
    let high: u32;

    unsafe {
        asm!("rdmsr", in("ecx") addr, out("eax") low, out("edx") high);
    }

    ((high as u64) << 32) | (low as u64)
}

pub fn write_msr(addr: u32, value: u64) {
    let low = value as u32;
    let high = (value >> 32) as u32;

    unsafe {
        asm!("wrmsr", in("ecx") addr, in("eax") low, in("edx") high);
    }
}

pub fn read_xcr0() -> u64 {
    let value;
    unsafe {
        asm!("xgetbv", out("rax") value);
    }
    value
}

pub fn write_xcr0(value: u64) {
    unsafe {
        asm!("xsetbv", in("rax") value);
    }
}
