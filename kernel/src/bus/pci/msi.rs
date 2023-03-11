use crate::arch::register::msi::{MsiMessageAddressField, MsiMessageDataField};

use super::conf_space;
use modular_bitfield::{bitfield, specifiers::*, BitfieldSpecifier};

#[bitfield]
#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[repr(C)]
pub struct MsiMessageControlField
{
    pub is_enable: bool,
    pub multiple_msg_capable: B3,
    pub multiple_msg_enable: B3,
    pub is_64bit: bool,
    pub per_vec_masking: bool,
    #[skip]
    reserved: B7,
}

#[bitfield]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct MsiCapabilityField
{
    pub cap_id: B8,
    pub next_ptr: B8,
    pub msg_ctrl: MsiMessageControlField,
    pub msg_addr_low: MsiMessageAddressField,
    pub msg_addr_high: B32,
    pub msg_data: MsiMessageDataField,
    #[skip]
    reserved: B16,
}

impl MsiCapabilityField
{
    pub fn read(bus: usize, device: usize, func: usize, caps_ptr: usize) -> Option<Self>
    {
        let caps_ptr = caps_ptr as usize;
        let data = [
            conf_space::read_conf_space(bus, device, func, caps_ptr),
            conf_space::read_conf_space(bus, device, func, caps_ptr + 4),
            conf_space::read_conf_space(bus, device, func, caps_ptr + 8),
            conf_space::read_conf_space(bus, device, func, caps_ptr + 12),
        ];

        if data.iter().filter(|&d| d.is_none()).count() != 0
        {
            return None;
        }

        let data = data.map(|d| d.unwrap());
        let field = unsafe { data.align_to::<Self>() }.1[0];
        return Some(field);
    }
}
