use log::warn;

use super::asm;

const ISA_DEBUG_EXIT_PORT: u32 = 0xf4;

pub fn exit(exit_code: u32) {
    asm::out32(ISA_DEBUG_EXIT_PORT, exit_code);

    // if QEMU, unreachable
    warn!("Failed to exit QEMU");
}
