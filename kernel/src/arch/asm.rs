use core::{arch::asm, mem::transmute};

pub fn hlt()
{
    unsafe {
        asm!("hlt");
    }
}

pub fn cli()
{
    unsafe {
        asm!("cli");
    }
}

pub fn sti()
{
    unsafe {
        asm!("sti");
    }
}

pub fn int3()
{
    unsafe {
        asm!("int3");
    }
}

pub fn out8(port: u16, data: u8)
{
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") data);
    }
}

pub fn in8(port: u16) -> u8
{
    let mut data: u8;
    unsafe {
        asm!("in al, dx", out("al") data, in("dx") port);
    }
    return data;
}

pub fn out32(port: u32, data: u32)
{
    unsafe {
        asm!("out dx, eax", in("edx") port, in("eax") data);
    }
}

pub fn in32(port: u32) -> u32
{
    let mut data: u32;
    unsafe {
        asm!("in eax, dx", out("eax") data, in("edx") port);
    }
    return data;
}

pub fn set_ds(value: u16)
{
    unsafe {
        asm!("mov ds, {}", in(reg) value);
    }
}

pub fn set_es(value: u16)
{
    unsafe {
        asm!("mov es, {}", in(reg) value);
    }
}

pub fn set_fs(value: u16)
{
    unsafe {
        asm!("mov fs, {}", in(reg) value);
    }
}

pub fn set_gs(value: u16)
{
    unsafe {
        asm!("mov gs, {}", in(reg) value);
    }
}

pub fn set_ss(value: u16)
{
    unsafe {
        asm!("mov ss, {}", in(reg) value);
    }
}

pub fn set_cs(value: u16)
{
    // TODO
    unsafe {
        asm!("push {}", in(reg) value);
        asm!("lea {tmp}, [1f + rip]", "push {tmp}", tmp = lateout(reg) _);
        asm!("retfq", "1:");
    }
}

#[repr(C, packed(2))]
#[derive(Debug)]
pub struct DescriptorTableArgs
{
    pub limit: u16,
    pub base: u64,
}

pub fn sidt() -> DescriptorTableArgs
{
    let mut data = [0; 10];
    unsafe {
        asm!("sidt [{}]", in(reg) &mut data);
    }
    return unsafe { transmute::<[u8; 10], DescriptorTableArgs>(data) };
}

pub fn lidt(desc_table_args: &DescriptorTableArgs)
{
    unsafe {
        asm!("lidt [{}]", in(reg) desc_table_args);
    }
}

pub fn sgdt() -> DescriptorTableArgs
{
    let mut data = [0; 10];
    unsafe {
        asm!("sgdt [{}]", in(reg) &mut data);
    }
    return unsafe { transmute::<[u8; 10], DescriptorTableArgs>(data) };
}

pub fn lgdt(desc_table_args: &DescriptorTableArgs)
{
    unsafe {
        asm!("lgdt [{}]", in(reg) desc_table_args);
    }
}

pub fn read_cs() -> u16
{
    let mut cs = 0;
    unsafe {
        asm!("mov {}, cs", out(reg) cs);
    }
    return cs;
}

pub fn read_cr0() -> u64
{
    let mut cr0 = 0;
    unsafe {
        asm!("mov {}, cr0", out(reg) cr0);
    }
    return cr0;
}

pub fn write_cr0(value: u64)
{
    unsafe {
        asm!("mov cr3, {}", in(reg) value);
    }
}

pub fn read_cr2() -> u64
{
    let mut cr2 = 0;
    unsafe {
        asm!("mov {}, cr2", out(reg) cr2);
    }
    return cr2;
}

pub fn read_cr3() -> u64
{
    let mut cr3 = 0;
    unsafe {
        asm!("mov {}, cr3", out(reg) cr3);
    }
    return cr3;
}

pub fn write_cr3(value: u64)
{
    unsafe {
        asm!("mov cr3, {}", in(reg) value);
    }
}
