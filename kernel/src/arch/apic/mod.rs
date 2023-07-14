use crate::arch::addr::*;

const LOCAL_APIC_REG_ADDR: u64 = 0xfee00020;

pub mod timer;

pub fn read_local_apic_id() -> u8 {
    let addr = VirtualAddress::new(LOCAL_APIC_REG_ADDR);
    let reg: u32 = addr.read_volatile();
    return (reg >> 24) as u8;
}
