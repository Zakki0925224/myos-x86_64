use super::context::Context;
use core::arch::asm;

pub fn hlt() {
    unsafe {
        asm!("hlt");
    }
}

pub fn cli() {
    unsafe {
        asm!("cli");
    }
}

pub fn sti() {
    unsafe {
        asm!("sti");
    }
}

pub fn disabled_int_func<F: FnMut()>(mut func: F) {
    cli();
    func();
    sti();
}

pub fn int3() {
    unsafe {
        asm!("int3");
    }
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
    let mut data: u8;
    unsafe {
        asm!(
            "in al, dx",
            out("al") data,
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
    let mut data: u32;
    unsafe {
        asm!(
            "in eax, dx",
            out("eax") data,
            in("edx") port
        );
    }
    data
}

pub fn set_ds(value: u16) {
    unsafe {
        asm!(
            "mov ds, ax",
            in("ax") value
        );
    }
}

pub fn set_es(value: u16) {
    unsafe {
        asm!(
            "mov es, ax",
            in("ax") value
        );
    }
}

pub fn set_fs(value: u16) {
    unsafe {
        asm!(
            "mov fs, ax",
            in("ax") value
        );
    }
}

pub fn set_gs(value: u16) {
    unsafe {
        asm!(
            "mov gs, ax",
            in("ax") value
        );
    }
}

pub fn set_ss(value: u16) {
    unsafe {
        asm!(
            "mov ss, ax",
            in("ax") value
        );
    }
}

pub fn set_cs(value: u16) {
    // reference: https://github.com/hikalium/wasabi/blob/main/os/src/x86_64.rs
    unsafe {
        asm!(
            "lea rax, [rip + 1f]",
            "push cx",
            "push rax",
            "ljmp [rsp]",
            "1:",
            "add rsp, 8 + 2",
            in("cx") value
        );
    }
}

#[repr(C, packed(2))]
#[derive(Debug)]
pub struct DescriptorTableArgs {
    pub limit: u16,
    pub base: u64,
}

impl Default for DescriptorTableArgs {
    fn default() -> Self {
        Self { limit: 0, base: 0 }
    }
}

pub fn sidt() -> DescriptorTableArgs {
    let mut args = DescriptorTableArgs::default();
    unsafe {
        asm!("sidt [{}]", in(reg) &mut args);
    }

    args
}

pub fn lidt(desc_table_args: &DescriptorTableArgs) {
    unsafe {
        asm!("lidt [{}]", in(reg) desc_table_args);
    }
}

pub fn sgdt() -> DescriptorTableArgs {
    let mut args = DescriptorTableArgs::default();
    unsafe {
        asm!("sgdt [{}]", in(reg) &mut args);
    }

    args
}

pub fn lgdt(desc_table_args: &DescriptorTableArgs) {
    unsafe {
        asm!("lgdt [{}]", in(reg) desc_table_args);
    }
}

pub fn read_cs() -> u16 {
    let mut cs = 0;
    unsafe {
        asm!("mov {0:r}, cs", out(reg) cs);
    }
    cs
}

pub fn read_cr0() -> u64 {
    let mut cr0 = 0;
    unsafe {
        asm!("mov {}, cr0", out(reg) cr0);
    }
    cr0
}

pub fn write_cr0(value: u64) {
    unsafe {
        asm!("mov cr3, {}", in(reg) value);
    }
}

pub fn read_cr2() -> u64 {
    let mut cr2 = 0;
    unsafe {
        asm!("mov {}, cr2", out(reg) cr2);
    }
    cr2
}

pub fn read_cr3() -> u64 {
    let mut cr3 = 0;
    unsafe {
        asm!("mov {}, cr3", out(reg) cr3);
    }
    cr3
}

pub fn write_cr3(value: u64) {
    unsafe {
        asm!("mov cr3, {}", in(reg) value);
    }
}

pub fn read_msr(addr: u32) -> u64 {
    let mut low: u32 = 0;
    let mut high: u32 = 0;

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

// TODO
pub fn save_context() -> Context {
    let mut ctx = Context::default();

    ctx.cr3 = read_cr3();
    //ctx.rip =
    //ctx.rflags =
    ctx.cs = read_cs() as u64;

    unsafe {
        asm!(
            "mov {}, ss",
            "mov {}, fs",
            "mov {}, gs",
            "mov {}, rax",
            "mov {}, rbx",
            "mov {}, rcx",
            "mov {}, rdx",
            "mov {}, rdi",
            "mov {}, rsi",
            "mov {}, rsp",
            "mov {}, rbp",
            "mov {}, r8",
            "mov {}, r9",
            "mov {}, r10",
            "mov {}, r11",
            "mov {}, r12",
            "mov {}, r13",
            "mov {}, r14",
            "mov {}, r15",
            "fxsave [{}]",
            out(reg) ctx.ss,
            out(reg) ctx.fs,
            out(reg) ctx.gs,
            out(reg) ctx.rax,
            out(reg) ctx.rbx,
            out(reg) ctx.rcx,
            out(reg) ctx.rdx,
            out(reg) ctx.rdi,
            out(reg) ctx.rsi,
            out(reg) ctx.rsp,
            out(reg) ctx.rbp,
            out(reg) ctx.r8,
            out(reg) ctx.r9,
            out(reg) ctx.r10,
            out(reg) ctx.r11,
            out(reg) ctx.r12,
            out(reg) ctx.r13,
            out(reg) ctx.r14,
            out(reg) ctx.r15,
            in(reg) &mut ctx.fpu_context
        );
    }

    ctx
}
