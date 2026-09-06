use crate::{
    device::{tty, Driver, DeviceInfo},
    error::Result,
    fs::vfs,
    kinfo,
    sync::mutex::Mutex,
    util::keyboard::{key_event::*, scan_code::KeyCode},
};
use alloc::collections::vec_deque::VecDeque;

const NAME: &str = "keyboard";

static KEYBOARD_DRIVER: Mutex<KeyboardDriver> = Mutex::new(KeyboardDriver::new());

struct KeyboardDriver {
    queue: VecDeque<KeyEvent>,
}

impl KeyboardDriver {
    const fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }
}

impl Driver for KeyboardDriver {
    fn info(&self) -> DeviceInfo {
        DeviceInfo::new(NAME)
    }

    fn attach(&mut self) -> Result<()> {
        Ok(())
    }

    fn poll(&mut self) -> Result<()> {
        loop {
            let event = match self.queue.pop_front() {
                Some(e) => e,
                None => return Ok(()),
            };

            if event.state != KeyState::Pressed {
                continue;
            }

            match event.code {
                KeyCode::CursorUp => tty::input_str("\x1b[A")?,
                KeyCode::CursorDown => tty::input_str("\x1b[B")?,
                KeyCode::CursorRight => tty::input_str("\x1b[C")?,
                KeyCode::CursorLeft => tty::input_str("\x1b[D")?,
                _ => {
                    if let Some(c) = event.c {
                        tty::input(c)?;
                    }
                }
            }
        }
    }
}

pub fn probe_and_attach() -> Result<()> {
    KEYBOARD_DRIVER.try_lock()?.attach()?;
    vfs::add_dev(&KEYBOARD_DRIVER)?;
    kinfo!("{}: Attached!", NAME);

    Ok(())
}

pub fn push_key_event(event: KeyEvent) -> Result<()> {
    KEYBOARD_DRIVER.try_lock()?.queue.push_back(event);

    Ok(())
}

pub fn poll_normal() -> Result<()> {
    KEYBOARD_DRIVER.try_lock()?.poll()
}
