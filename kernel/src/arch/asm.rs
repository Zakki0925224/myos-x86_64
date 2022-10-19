use core::arch::asm;

pub fn hlt() { unsafe { asm!("hlt") } }

pub fn cli() { unsafe { asm!("cli") } }

pub fn sti() { unsafe { asm!("sti") } }

pub fn out8(port: u16, data: u8) { unsafe { asm!("out dx, al", in("dx") port, in("al") data) } }

pub fn in8(port: u16) -> u8
{
    let mut data: u8;
    unsafe { asm!("in al, dx", out("al") data, in("dx") port) }
    return data;
}
