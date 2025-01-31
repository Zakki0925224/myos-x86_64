use super::eth::{EtherType, EthernetAddress};
use core::net::Ipv4Addr;

#[derive(Debug)]
pub struct ArpPacket {
    hardware_ty: [u8; 2],
    protocol_ty: EtherType,
    hardware_len: u8,
    protocol_len: u8,
    operation: [u8; 2],
    sender_hardware_addr: EthernetAddress,
    sender_protocol_addr: Ipv4Addr,
    target_hardware_addr: EthernetAddress,
    target_protocol_addr: Ipv4Addr,
}

impl ArpPacket {
    pub fn new(data: &[u8]) -> Self {
        Self {
            hardware_ty: [data[0], data[1]],
            protocol_ty: [data[2], data[3]].into(),
            hardware_len: data[4],
            protocol_len: data[5],
            operation: [data[6], data[7]],
            sender_hardware_addr: [data[8], data[9], data[10], data[11], data[12], data[13]].into(),
            sender_protocol_addr: [data[14], data[15], data[16], data[17]].into(),
            target_hardware_addr: [data[18], data[19], data[20], data[21], data[22], data[23]]
                .into(),
            target_protocol_addr: [data[24], data[25], data[26], data[27]].into(),
        }
    }
}
