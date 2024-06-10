use self::{key_event::KeyEvent, key_map::KeyMap};
use crate::{
    arch::addr::IoPortAddress,
    device::ps2_keyboard::{
        key_event::{KeyState, ModifierKeysState},
        key_map::ANSI_US_104_KEY_MAP,
    },
    error::Result,
    util::{
        fifo::Fifo,
        mutex::{Mutex, MutexError},
    },
};

mod key_event;
mod key_map;
mod scan_code;

const PS2_DATA_REG_ADDR: IoPortAddress = IoPortAddress::new(0x60);
const PS2_CMD_AND_STATE_REG_ADDR: IoPortAddress = IoPortAddress::new(0x64);

static mut KEYBOARD: Mutex<Keyboard> = Mutex::new(Keyboard::new(ANSI_US_104_KEY_MAP));

struct Keyboard {
    key_map: KeyMap,
    mod_keys_state: ModifierKeysState,
    data_buf: Fifo<u8, 128>,
    data_0: Option<u8>,
    data_1: Option<u8>,
    data_2: Option<u8>,
    data_3: Option<u8>,
    data_4: Option<u8>,
    data_5: Option<u8>,
}

impl Keyboard {
    pub const fn new(key_map: KeyMap) -> Self {
        Self {
            key_map,
            mod_keys_state: ModifierKeysState {
                shift: false,
                ctrl: false,
                gui: false,
                alt: false,
            },
            data_buf: Fifo::new(0),
            data_0: None,
            data_1: None,
            data_2: None,
            data_3: None,
            data_4: None,
            data_5: None,
        }
    }

    pub fn input(&mut self, data: u8) -> Result<()> {
        if self.data_buf.enqueue(data).is_err() {
            self.data_buf.reset_ptr();
            self.data_buf.enqueue(data)?;
        }

        Ok(())
    }

    pub fn get_event(&mut self) -> Result<Option<KeyEvent>> {
        let data = self.data_buf.dequeue()?;

        if self.data_0.is_none() {
            self.data_0 = Some(data);
        } else if self.data_1.is_none() {
            self.data_1 = Some(data);
        } else if self.data_2.is_none() {
            self.data_2 = Some(data);
        } else if self.data_3.is_none() {
            self.data_3 = Some(data);
        } else if self.data_4.is_none() {
            self.data_4 = Some(data);
        } else if self.data_5.is_none() {
            self.data_5 = Some(data);
        } else {
            self.data_0 = Some(data);
            self.data_1 = None;
            self.data_2 = None;
            self.data_3 = None;
            self.data_4 = None;
            self.data_5 = None;
        }

        if let (
            Some(data_0),
            Some(data_1),
            Some(data_2),
            Some(data_3),
            Some(data_4),
            Some(data_5),
        ) = (
            self.data_0,
            self.data_1,
            self.data_2,
            self.data_3,
            self.data_4,
            self.data_5,
        ) {
            let code = [data_0, data_1, data_2, data_3, data_4, data_5];
            let key_map = match self.key_map {
                KeyMap::AnsiUs104(map) => map,
            };

            for scan_code in key_map {
                let key_code = scan_code.key_code;

                let key_state = if scan_code.pressed == code {
                    KeyState::Pressed
                } else if scan_code.released == code {
                    KeyState::Released
                } else {
                    continue;
                };

                let ModifierKeysState {
                    shift: prev_shift,
                    ctrl: prev_ctrl,
                    gui: prev_gui,
                    alt: prev_alt,
                } = self.mod_keys_state;

                self.mod_keys_state = ModifierKeysState {
                    shift: key_code.is_shift() || prev_shift,
                    ctrl: key_code.is_ctrl() || prev_ctrl,
                    gui: key_code.is_gui() || prev_gui,
                    alt: key_code.is_alt() || prev_alt,
                };

                let ascii_code = match self.mod_keys_state.shift {
                    true => scan_code.on_shift_ascii_code,
                    false => scan_code.ascii_code,
                };

                let key_event = KeyEvent {
                    code: key_code,
                    state: key_state,
                    ascii: ascii_code,
                };

                return Ok(Some(key_event));
            }
        }

        Ok(None)
    }
}

pub fn init() {
    PS2_CMD_AND_STATE_REG_ADDR.out8(0x60); // write configuration byte
    wait_ready();
    PS2_DATA_REG_ADDR.out8(0x47); // enable interrupt
    wait_ready();

    PS2_CMD_AND_STATE_REG_ADDR.out8(0x20); // read configuration byte
    wait_ready();
}

pub fn receive() -> Result<()> {
    let data = PS2_DATA_REG_ADDR.in8();
    if let Ok(mut keyboard) = unsafe { KEYBOARD.try_lock() } {
        return keyboard.input(data);
    }

    Err(MutexError::Locked.into())
}

pub fn get_event() -> Result<Option<KeyEvent>> {
    if let Ok(mut keyboard) = unsafe { KEYBOARD.try_lock() } {
        return keyboard.get_event();
    }

    Err(MutexError::Locked.into())
}

fn wait_ready() {
    while PS2_CMD_AND_STATE_REG_ADDR.in8() & 0x2 != 0 {
        continue;
    }
}
