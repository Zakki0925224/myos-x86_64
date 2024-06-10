use crate::util::ascii::AsciiCode;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[allow(dead_code)]
pub enum KeyCode {
    Esc,          // p: 0x01, r: 0x81
    F1,           // p: 0x3b, r: 0xbb
    F2,           // p: 0x3c, r: 0xbc
    F3,           // p: 0x3d, r: 0xbd
    F4,           // p: 0x3e, r: 0xbe
    F5,           // p: 0x3f, r: 0xbf
    F6,           // p: 0x40, r: 0xc0
    F7,           // p: 0x41, r: 0xc1
    F8,           // p: 0x42, r: 0xc2
    F9,           // p: 0x43, r: 0xc3
    F10,          // p: 0x44, r: 0xc4
    F11,          // p: 0x57, r: 0xd7
    F12,          // p: 0x58, r: 0xd8
    PrintScreen,  // p: 0xe0, 0x2a, 0xe0, 0x37, r: 0xe0, 0xb7, 0xe0, 0xaa
    ScrollLock,   // p: 0x46, r: 0xc6
    Pause,        // p: 0xe1, 0x1d, 0x45, 0x9d, 0xc5, r: none
    Insert,       // p: 0xe0, 0x52, r: 0xe0, 0xd2
    Home,         // p: 0xe0, 0x47, r: 0xe0, 0xc7
    PageUp,       // p: 0xe0, 0x49, r: 0xe0, 0xc9
    Delete,       // p: 0xe0, 0x53, r: 0xe0, 0xd3
    End,          // p: 0xe0, 0x4f, r: 0xe0, 0xcf
    PageDown,     // p: 0xe0, 0x51, r: 0xe0, 0xd1
    CursorRight,  // p: 0xe0, 0x4d, r: 0xe0, 0xcd
    CursorLeft,   // p: 0xe0, 0x4b, r: 0xe0, 0xcb
    CursorDown,   // p: 0xe0, 0x50, r: 0xe0, 0xd0
    CursorUp,     // p: 0xe0, 0x48, r: 0xe0, 0xc8
    NumLock,      // p: 0x45, r: 0xc5
    KpDivide,     // p: 0xe0, 0x35, r: 0xe0, 0xb5
    KpMultiply,   // p: 0x37, r: 0xb7
    KpSubtract,   // p: 0x4a, r: 0xca
    KpAdd,        // p: 0x4e, r: 0xce
    KpEnter,      // p: 0xe0, 0x1c, r: 0xe0, 0x9c
    Kp1,          // p: 0x4f, r: 0xcf
    Kp2,          // p: 0x50, r: 0xd0
    Kp3,          // p: 0x51, r: 0xd1
    Kp4,          // p: 0x4b, r: 0xcb
    Kp5,          // p: 0x4c, r: 0xcc
    Kp6,          // p: 0x4d, r: 0xcd
    Kp7,          // p: 0x47, r: 0xc7
    Kp8,          // p: 0x48, r: 0xc8
    Kp9,          // p: 0x49, r: 0xc9
    Kp0,          // p: 0x52, r: 0xd2
    KpPeriod,     // p: 0x53, r: 0xd3
    LCtrl,        // p: 0x1d, r: 0x9d
    LGui,         // p: 0xe0, 0x5b, r: 0xe0, 0xdb
    LAlt,         // p: 0x38, r: 0xb8
    Space,        // p: 0x39, r: 0xb9
    RGui,         // p: 0xe0, 0x5c, r: 0xe0, 0xdc
    RAlt,         // p: 0xe0, 0x38, r: 0xe0, 0xb8
    Apps,         // p: 0xe0, 0x6d, r: 0xe0, 0xdd
    RCtrl,        // p: 0xe0, 0x1d, r: 0xe0, 0x9d
    LShift,       // p: 0x2a, r: 0xaa
    CapsLock,     // p: 0x3a, r: 0xba
    Tab,          // p: 0x0f, r: 0x8f
    Backspace,    // p: 0x0e, r: 0x8e
    Enter,        // p: 0x1c, r: 0x9c
    RShift,       // p: 0x36, r: 0xb6
    Num1,         // p: 0x02, r: 0x82
    Num2,         // p: 0x03, r: 0x83
    Num3,         // p: 0x04, r: 0x84
    Num4,         // p: 0x05, r: 0x85
    Num5,         // p: 0x06, r: 0x86
    Num6,         // p: 0x07, r: 0x87
    Num7,         // p: 0x08, r: 0x88
    Num8,         // p: 0x09, r: 0x89
    Num9,         // p: 0x0a, r: 0x8a
    Num0,         // p: 0x0b, r: 0x8b
    A,            // p: 0x1e, r: 0x9e
    B,            // p: 0x30, r: 0xb0
    C,            // p: 0x2e, r: 0xae
    D,            // p: 0x20, r: 0xa0
    E,            // p: 0x12, r: 0x92
    F,            // p: 0x21, r: 0xa1
    G,            // p: 0x22, r: 0xa2
    H,            // p: 0x23, r: 0xa3
    I,            // p: 0x17, r: 0x97
    J,            // p: 0x24, r: 0xa4
    K,            // p: 0x25, r: 0xa5
    L,            // p: 0x26, r: 0xa6
    M,            // p: 0x32, r: 0xb2
    N,            // p: 0x31, r: 0xb1
    O,            // p: 0x18, r: 0x98
    P,            // p: 0x19, r: 0x99
    Q,            // p: 0x10, r: 0x90
    R,            // p: 0x13, r: 0x93
    S,            // p: 0x1f, r: 0x9f
    T,            // p: 0x14, r: 0x94
    U,            // p: 0x16, r: 0x96
    V,            // p: 0x2f, r: 0xaf
    W,            // p: 0x11, r: 0x91
    X,            // p: 0x2d, r: 0xad
    Y,            // p: 0x15, r: 0x95
    Z,            // p: 0x2c, r: 0xac
    Backtick,     // ` p: 0x29, r: 0xa9
    Subtract,     // - p: 0x0c, r: 0x8c
    Equal,        // = p: 0x0d, r: 0x8d
    BracketLeft,  // [ p: 0x1a, r: 0x9a
    BracketRight, // ] p: 0x1b, r: 0x9b
    Backslash,    // \ p: 0x2b, r: 0xab
    Semicolon,    // ; p: 0x27, r: 0xa7
    Quote,        // ' p: 0x28, r: 0xa8
    Comma,        // , p: 0x33, r: 0xb3
    Period,       // . p: 0x34, r: 0xb4
    Slash,        // / p: 0x35, r: 0xb5
}

impl KeyCode {
    pub fn is_shift(&self) -> bool {
        *self == KeyCode::LShift || *self == KeyCode::RShift
    }

    pub fn is_ctrl(&self) -> bool {
        *self == KeyCode::LCtrl || *self == KeyCode::RCtrl
    }

    pub fn is_gui(&self) -> bool {
        *self == KeyCode::LGui || *self == KeyCode::RGui
    }

    pub fn is_alt(&self) -> bool {
        *self == KeyCode::LAlt || *self == KeyCode::RAlt
    }
}

// https://wiki.osdev.org/PS2_Keyboard
// scan code set 1
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct ScanCode {
    pub key_code: KeyCode,
    pub ascii_code: Option<AsciiCode>,
    pub on_shift_ascii_code: Option<AsciiCode>,
    pub pressed: [u8; 6],
    pub released: [u8; 6],
}
