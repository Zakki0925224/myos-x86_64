use modular_bitfield::{bitfield, specifiers::*};

#[bitfield]
#[derive(Debug)]
#[repr(C)]
pub struct InputContext
{
    #[skip]
    reserved0: B2,
    pub drop_context_flags: B30,
    pub add_context_flags: B32,
    #[skip]
    reserved1: B32,
    #[skip]
    reserved2: B32,
    #[skip]
    reserved3: B32,
    #[skip]
    reserved4: B32,
    #[skip]
    reserved5: B32,
    pub conf_value: B8,
    pub interface_num: B8,
    pub alternate_setting: B8,
    #[skip]
    reserved6: B8,
}
