use crate::arch::addr::IoPortAddress;
use log::warn;

pub fn exit(exit_code: u32) {
    // ISA debug exit
    IoPortAddress::new(0xf4).out32(exit_code);

    // if QEMU, unreachable
    warn!("Failed to exit QEMU");
}
