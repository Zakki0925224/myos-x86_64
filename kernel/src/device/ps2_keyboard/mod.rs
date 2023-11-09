use crate::{arch::addr::IoPortAddress, println};
use log::info;

use self::{key_event::KeyEvent, key_map::KeyMap};

mod key_event;
mod key_map;
mod scan_code;

const KBD_DATA_REG_ADDR: IoPortAddress = IoPortAddress::new(0x60);
const KBD_CMD_AND_STATE_REG_ADDR: IoPortAddress = IoPortAddress::new(0x64);

struct Keyboard {
    key_map: KeyMap,
    key_event: Option<KeyEvent>,
}

impl Keyboard {
    pub fn new(key_map: KeyMap) -> Self {
        Self {
            key_map,
            key_event: None,
        }
    }
}

pub fn init() {
    wait_ready();
    KBD_CMD_AND_STATE_REG_ADDR.out8(0x60); // wite mode
    wait_ready();
    KBD_CMD_AND_STATE_REG_ADDR.out8(0x47); // kbc mode

    info!("ps2 kbd: Initialized");
}

pub fn receive() {
    let data = KBD_DATA_REG_ADDR.in8();
    println!("ps2 kbd: 0x{:x}", data);
}

fn wait_ready() {
    while KBD_CMD_AND_STATE_REG_ADDR.in8() & 0x2 != 0 {
        continue;
    }
}
