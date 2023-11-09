use log::warn;

use super::addr::PhysicalAddress;

pub fn exit(exit_code: u32) {
    // ISA debug exit
    PhysicalAddress::new(0xf4).out32(exit_code);

    // if QEMU, unreachable
    warn!("Failed to exit QEMU");
}
