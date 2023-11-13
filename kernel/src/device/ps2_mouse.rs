use log::info;

use crate::{arch::addr::IoPortAddress, println};

const PS2_DATA_REG_ADDR: IoPortAddress = IoPortAddress::new(0x60);
const PS2_CMD_AND_STATE_REG_ADDR: IoPortAddress = IoPortAddress::new(0x64);

pub fn init() {
    PS2_CMD_AND_STATE_REG_ADDR.out8(0xd4); // send next wrote byte to ps/2 secondary port
    wait_ready();

    PS2_DATA_REG_ADDR.out8(0xff); // init mouse
    wait_ready();

    PS2_CMD_AND_STATE_REG_ADDR.out8(0xd4);
    wait_ready();

    PS2_DATA_REG_ADDR.out8(0xf4); // start streaming
    wait_ready();

    info!("ps2 mouse: Initialized");
}

pub fn receive() {
    let data = PS2_DATA_REG_ADDR.in8();
    println!("ps2 mouse: 0x{:x}", data);
}

fn wait_ready() {
    while PS2_CMD_AND_STATE_REG_ADDR.in8() & 0x2 != 0 {
        continue;
    }
}
