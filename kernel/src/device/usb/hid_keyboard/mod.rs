use modular_bitfield::{bitfield, specifiers::B8};

#[bitfield]
#[derive(Debug)]
#[repr(C)]
pub struct InputData {
    modifier_key: B8,
    #[skip]
    reserved: B8,
    key_code1: B8,
    key_code2: B8,
    key_code3: B8,
    key_code4: B8,
    key_code5: B8,
    key_code6: B8,
}
