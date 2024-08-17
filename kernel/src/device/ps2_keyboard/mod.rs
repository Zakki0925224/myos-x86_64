use alloc::string::String;
use log::{error, info};

use self::{key_event::KeyEvent, key_map::KeyMap};
use crate::{
    arch::{self, addr::IoPortAddress},
    device::ps2_keyboard::{
        key_event::{KeyState, ModifierKeysState},
        key_map::ANSI_US_104_KEY_MAP,
    },
    error::{Error, Result},
    idt::{self, GateType, InterruptHandler},
    print, println,
    util::{ascii::AsciiCode, fifo::Fifo, mutex::Mutex},
};

use super::{console, DeviceDriverFunction, DeviceDriverInfo};

pub mod key_event;
mod key_map;
mod scan_code;

const PS2_DATA_REG_ADDR: IoPortAddress = IoPortAddress::new(0x60);
const PS2_CMD_AND_STATE_REG_ADDR: IoPortAddress = IoPortAddress::new(0x64);

static mut PS2_KBD_DRIVER: Mutex<Ps2KeyboardDriver> =
    Mutex::new(Ps2KeyboardDriver::new(ANSI_US_104_KEY_MAP));

struct Ps2KeyboardDriver {
    device_driver_info: DeviceDriverInfo,
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

impl Ps2KeyboardDriver {
    const fn new(key_map: KeyMap) -> Self {
        Self {
            device_driver_info: DeviceDriverInfo::new("ps2-kbd"),
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

    fn input(&mut self, data: u8) -> Result<()> {
        if self.data_buf.enqueue(data).is_err() {
            self.data_buf.reset_ptr();
            self.data_buf.enqueue(data)?;
        }

        //println!("{:?}", self.data_buf.get_buf_ref());

        Ok(())
    }

    fn get_event(&mut self) -> Result<Option<KeyEvent>> {
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
            self.clear_data();
            self.data_0 = Some(data);
        }

        let code = [
            self.data_0.unwrap_or(0),
            self.data_1.unwrap_or(0),
            self.data_2.unwrap_or(0),
            self.data_3.unwrap_or(0),
            self.data_4.unwrap_or(0),
            self.data_5.unwrap_or(0),
        ];
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

            // println!("{:?}", code);
            // println!("{:?}, {:?}", key_code, key_state);

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

            self.clear_data();
            return Ok(Some(key_event));
        }

        Ok(None)
    }

    fn clear_data(&mut self) {
        self.data_0 = None;
        self.data_1 = None;
        self.data_2 = None;
        self.data_3 = None;
        self.data_4 = None;
        self.data_5 = None;
    }

    fn wait_ready(&self) {
        while PS2_CMD_AND_STATE_REG_ADDR.in8() & 0x2 != 0 {
            continue;
        }
    }
}

impl DeviceDriverFunction for Ps2KeyboardDriver {
    type PollNormalOutput = Option<KeyEvent>;
    type PollInterruptOutput = ();

    fn get_device_driver_info(&self) -> Result<DeviceDriverInfo> {
        Ok(self.device_driver_info.clone())
    }

    fn probe(&mut self) -> Result<()> {
        Ok(())
    }

    fn attach(&mut self) -> Result<()> {
        PS2_CMD_AND_STATE_REG_ADDR.out8(0x60); // write configuration byte
        self.wait_ready();
        PS2_DATA_REG_ADDR.out8(0x47); // enable interrupt
        self.wait_ready();

        PS2_CMD_AND_STATE_REG_ADDR.out8(0x20); // read configuration byte
        self.wait_ready();

        self.device_driver_info.attached = true;
        Ok(())
    }

    fn poll_normal(&mut self) -> Result<Self::PollNormalOutput> {
        if !self.device_driver_info.attached {
            return Err(Error::Failed("Device driver is not attached"));
        }

        self.get_event()
    }

    fn poll_int(&mut self) -> Result<Self::PollInterruptOutput> {
        if !self.device_driver_info.attached {
            return Err(Error::Failed("Device driver is not attached"));
        }

        let data = PS2_DATA_REG_ADDR.in8();
        self.input(data)?;

        Ok(())
    }
}

pub fn get_device_driver_info() -> Result<DeviceDriverInfo> {
    let driver = unsafe { PS2_KBD_DRIVER.try_lock() }?;
    driver.get_device_driver_info()
}

pub fn probe_and_attach() -> Result<()> {
    arch::cli();
    {
        let mut driver = unsafe { PS2_KBD_DRIVER.try_lock() }?;
        driver.probe()?;
        driver.attach()?;
        info!("{}: Attached!", driver.get_device_driver_info()?.name);
    }
    arch::sti();

    Ok(())
}

pub fn poll_normal(is_prompt_mode: bool) -> Result<Option<String>> {
    let key_event;

    arch::cli();
    {
        let mut driver = unsafe { PS2_KBD_DRIVER.try_lock() }?;
        key_event = driver.poll_normal()?;
    }
    arch::sti();

    let key_event = match key_event {
        Some(e) => e,
        None => return Ok(None),
    };

    if key_event.state == KeyState::Released {
        return Ok(None);
    }

    let ascii_code = match key_event.ascii {
        Some(c) => c,
        None => return Ok(None),
    };

    match ascii_code {
        AsciiCode::CarriageReturn => {
            println!();
        }
        code => {
            print!("{}", code as u8 as char);
        }
    }

    let cmd = console::input(ascii_code)?;
    if !is_prompt_mode {
        return Ok(cmd);
    }

    let cmd = match cmd {
        Some(s) => s,
        None => return Ok(None),
    };

    if let Err(err) = console::exec_cmd(cmd) {
        error!("{:?}", err);
    }
    console::print_prompt();

    Ok(None)
}

pub extern "x86-interrupt" fn poll_int_ps2_kbd_driver() {
    if let Ok(mut driver) = unsafe { PS2_KBD_DRIVER.try_lock() } {
        let _ = driver.poll_int();
    }
    idt::pic_notify_end_of_int();
}
