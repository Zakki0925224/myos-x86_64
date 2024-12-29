use crate::arch::addr::*;

const LOCAL_APIC_REG_VIRT_ADDR: VirtualAddress = VirtualAddress::new(0xfee00020);

pub fn local_apic_id() -> u8 {
    let reg = unsafe { &*(LOCAL_APIC_REG_VIRT_ADDR.as_ptr() as *const u32) };
    (reg >> 24) as u8
}
