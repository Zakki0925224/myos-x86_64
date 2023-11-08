const KBD_DATA_REG_ADDR: u16 = 0x60;
const KBD_CMD_REG_ADDR: u16 = 0x64;
const KBD_STATE_REG_ADDR: u16 = 0x64;

use log::info;

use crate::{arch::asm, println};

pub fn init() {
    wait_ready();
    asm::out8(KBD_CMD_REG_ADDR, 0x60); // write mode
    wait_ready();
    asm::out8(KBD_CMD_REG_ADDR, 0x47); // kbc mode

    info!("ps2 kbd: Initialized");
}

pub fn receive() {
    let data = asm::in8(KBD_DATA_REG_ADDR);
    println!("ps2 kbd: 0x{:x}", data);
}

fn wait_ready() {
    while asm::in8(KBD_STATE_REG_ADDR) & 0x2 != 0 {
        continue;
    }
}
