use crate::{
    arch::addr::IoPortAddress,
    device::ps2_keyboard::{
        key_event::{KeyState, ModifierKeysState},
        key_map::ANSI_US_104_KEY_MAP,
        scan_code::KeyCode,
    },
    mem::buffer::fifo::Fifo,
    println,
};
use lazy_static::lazy_static;
use log::info;
use spin::Mutex;

use self::{key_event::KeyEvent, key_map::KeyMap};

mod key_event;
mod key_map;
mod scan_code;

const PS2_DATA_REG_ADDR: IoPortAddress = IoPortAddress::new(0x60);
const PS2_CMD_AND_STATE_REG_ADDR: IoPortAddress = IoPortAddress::new(0x64);

lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard> = Mutex::new(Keyboard::new(ANSI_US_104_KEY_MAP));
}

struct Keyboard {
    key_map: KeyMap,
    mod_keys_state: ModifierKeysState,
    key_event: Option<KeyEvent>,
    key_buf: Fifo<u8, 6>,
}

impl Keyboard {
    pub fn new(key_map: KeyMap) -> Self {
        Self {
            key_map,
            mod_keys_state: ModifierKeysState::default(),
            key_event: None,
            key_buf: Fifo::new(0),
        }
    }

    pub fn input(&mut self, data: u8) {
        let map = match self.key_map {
            KeyMap::AnsiUs104(map) => map,
        };

        if self.key_buf.enqueue(data).is_err() {
            self.reset_key_buf();
            self.key_buf.enqueue(data).unwrap();
        }

        let key_buf_ref = self.key_buf.get_buf_ref();

        for scan_code in map {
            let key_code = scan_code.key_code;

            if scan_code.pressed == *key_buf_ref {
                // pressed
                self.mod_keys_state = ModifierKeysState {
                    shift: key_code == KeyCode::LShift
                        || key_code == KeyCode::RShift
                        || self.mod_keys_state.shift,
                    ctrl: key_code == KeyCode::LCtrl
                        || key_code == KeyCode::RCtrl
                        || self.mod_keys_state.ctrl,
                    gui: key_code == KeyCode::LGui
                        || key_code == KeyCode::RGui
                        || self.mod_keys_state.gui,
                    alt: key_code == KeyCode::LAlt
                        || key_code == KeyCode::RAlt
                        || self.mod_keys_state.alt,
                };

                let ascii_code = match self.mod_keys_state.shift {
                    true => scan_code.on_shift_ascii_code,
                    false => scan_code.ascii_code,
                };

                self.key_event = Some(KeyEvent {
                    code: key_code,
                    state: KeyState::Pressed,
                    ascii: ascii_code,
                });
                break;
            } else if scan_code.released == *key_buf_ref {
                // released
                self.mod_keys_state = ModifierKeysState {
                    shift: key_code != KeyCode::LShift
                        && key_code != KeyCode::RShift
                        && self.mod_keys_state.shift,
                    ctrl: key_code != KeyCode::RCtrl
                        && key_code != KeyCode::LCtrl
                        && self.mod_keys_state.ctrl,
                    gui: key_code != KeyCode::LGui
                        && key_code != KeyCode::RGui
                        && self.mod_keys_state.gui,
                    alt: key_code != KeyCode::LAlt
                        && key_code != KeyCode::RAlt
                        && self.mod_keys_state.alt,
                };

                self.key_event = Some(KeyEvent {
                    code: key_code,
                    state: KeyState::Released,
                    ascii: None,
                });
                break;
            }
        }

        println!("{:?}", self.key_event);
        // if let Some(e) = self.key_event {
        //     if let Some(a) = e.ascii {
        //         println!("{}", a as u8 as char);
        //     }
        // }

        if (self.key_buf.len() == 1 && key_buf_ref[0] != 0xe0 && key_buf_ref[0] != 0xe1)
            || (self.key_buf.len() == 2
                && key_buf_ref[1] != 0x2a
                && key_buf_ref[1] != 0xb7
                && key_buf_ref[1] != 0x1d)
            || (self.key_buf.len() == 3 && key_buf_ref[2] != 0x45 && key_buf_ref[2] != 0xe0)
        {
            self.reset_key_buf();
            self.key_event = None;
        }
    }

    fn reset_key_buf(&mut self) {
        self.key_buf = Fifo::new(0);
    }
}

pub fn init() {
    PS2_CMD_AND_STATE_REG_ADDR.out8(0x60); // write configuration byte
    wait_ready();
    PS2_DATA_REG_ADDR.out8(0x47); // enable interrupt
    wait_ready();

    PS2_CMD_AND_STATE_REG_ADDR.out8(0x20); // read configuration byte
    wait_ready();
    let conf_byte = PS2_DATA_REG_ADDR.in8();
    println!("conf byte: 0x{:x}", conf_byte);

    info!("ps2 kbd: Initialized");
}

pub fn receive() {
    let data = PS2_DATA_REG_ADDR.in8();
    if let Some(mut keyboard) = KEYBOARD.try_lock() {
        keyboard.input(data);
    }
}

fn wait_ready() {
    while PS2_CMD_AND_STATE_REG_ADDR.in8() & 0x2 != 0 {
        continue;
    }
}
