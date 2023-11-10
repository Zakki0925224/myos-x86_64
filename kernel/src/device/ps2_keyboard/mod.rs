use crate::{
    arch::addr::IoPortAddress, device::ps2_keyboard::key_map::ANSI_US_104_KEY_MAP,
    mem::buffer::fifo::Fifo, println,
};
use lazy_static::lazy_static;
use log::info;
use spin::Mutex;

use self::{key_event::KeyEvent, key_map::KeyMap};

mod key_event;
mod key_map;
mod scan_code;

const KBD_DATA_REG_ADDR: IoPortAddress = IoPortAddress::new(0x60);
const KBD_CMD_AND_STATE_REG_ADDR: IoPortAddress = IoPortAddress::new(0x64);

lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard> = Mutex::new(Keyboard::new(ANSI_US_104_KEY_MAP));
}

struct Keyboard {
    key_map: KeyMap,
    key_event: Option<KeyEvent>,
    key_buf: Fifo<u8, 6>,
}

impl Keyboard {
    pub fn new(key_map: KeyMap) -> Self {
        Self {
            key_map,
            key_event: None,
            key_buf: Fifo::new(0),
        }
    }

    pub fn input(&mut self, data: u8) {
        //info!("ps2 kbd: 0x{:x}", data);

        let map = match self.key_map {
            KeyMap::AnsiUs104(map) => map,
        };

        if self.key_buf.enqueue(data).is_err() {
            self.key_buf.reset_ptr();
            self.key_buf.enqueue(data).unwrap();
        }

        for scan_code in map {
            if scan_code.pressed == *self.key_buf.get_buf_ref() {
                println!("{:?}", scan_code.key_code);
                self.key_buf.reset_ptr();
            } else if scan_code.released == *self.key_buf.get_buf_ref() {
                self.key_buf.reset_ptr();
            }
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
    KEYBOARD.lock().input(data);
}

fn wait_ready() {
    while KBD_CMD_AND_STATE_REG_ADDR.in8() & 0x2 != 0 {
        continue;
    }
}
