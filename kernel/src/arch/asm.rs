use core::arch::asm;

pub fn hlt() {
    unsafe { asm!("hlt") }
}

pub fn cli() {
    unsafe { asm!("cli") }
}

pub fn sti() {
    unsafe { asm!("sti") }
}

pub fn disabled_int_func<F: FnMut()>(mut func: F) {
    cli();
    func();
    sti();
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

#[repr(C, packed(2))]
#[derive(Debug, Default)]
pub struct DescriptorTableArgs {
    pub limit: u16,
    pub base: u64,
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
    let mut low: u32;
    let mut high: u32;

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
