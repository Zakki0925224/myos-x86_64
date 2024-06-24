use super::scan_code::KeyCode;
use crate::util::ascii::AsciiCode;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum KeyState {
    Pressed,
    Released,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct ModifierKeysState {
    pub shift: bool,
    pub ctrl: bool,
    pub gui: bool,
    pub alt: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub state: KeyState,
    pub ascii: Option<AsciiCode>,
}
