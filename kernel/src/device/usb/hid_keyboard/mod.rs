#[derive(Debug)]
#[repr(C)]
pub struct InputData {
    modifier_key: u8,
    _reserved: u8,
    key_code1: u8,
    key_code2: u8,
    key_code3: u8,
    key_code4: u8,
    key_code5: u8,
    key_code6: u8,
}
