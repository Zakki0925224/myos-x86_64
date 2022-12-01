use core::arch::asm;

pub fn hlt() { unsafe { asm!("hlt") } }

pub fn cli() { unsafe { asm!("cli") } }

pub fn sti() { unsafe { asm!("sti") } }

pub fn int3() { unsafe { asm!("int3") } }

pub fn out8(port: u16, data: u8) { unsafe { asm!("out dx, al", in("dx") port, in("al") data) } }

pub fn in8(port: u16) -> u8
{
    let mut data: u8;
    unsafe { asm!("in al, dx", out("al") data, in("dx") port) }
    return data;
}

#[repr(C, packed)]
pub struct DescriptorTableArgs
{
    pub base: u64,
    pub limit: u16,
}

pub fn sidt() -> DescriptorTableArgs
{
    let mut args_buf: [u8; 10] = [0; 10]; // 8bytes: limit, 2bytes: offset
    unsafe { asm!("sidt [{}]", in(reg) &mut args_buf) }

    let mut args = DescriptorTableArgs { base: 0, limit: 0 };
    args.base |= (args_buf[9] as u64) << 56;
    args.base |= (args_buf[8] as u64) << 48;
    args.base |= (args_buf[7] as u64) << 40;
    args.base |= (args_buf[6] as u64) << 32;
    args.base |= (args_buf[5] as u64) << 24;
    args.base |= (args_buf[4] as u64) << 16;
    args.base |= (args_buf[3] as u64) << 8;
    args.base |= (args_buf[2] as u64) << 0;
    args.limit = (args_buf[1] as u16) << 8 | args_buf[0] as u16;

    return args;
}

pub fn get_cs() -> u16
{
    let mut cs = 0;
    unsafe { asm!("mov {}, cs", out(reg) cs) }
    return cs;
}
