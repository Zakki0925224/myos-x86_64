use super::eth::{EtherType, EthernetAddress};
use crate::error::{Error, Result};
use core::net::Ipv4Addr;

pub enum ArpOperation {
    Request,
    Reply,
}

impl From<ArpOperation> for [u8; 2] {
    fn from(value: ArpOperation) -> [u8; 2] {
        match value {
            ArpOperation::Request => [0, 1],
            ArpOperation::Reply => [0, 2],
        }
    }
}

impl TryFrom<[u8; 2]> for ArpOperation {
    type Error = Error;

    fn try_from(value: [u8; 2]) -> Result<Self> {
        match value {
            [0, 1] => Ok(ArpOperation::Request),
            [0, 2] => Ok(ArpOperation::Reply),
            _ => Err(Error::Failed("Invalid ARP operation")),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ArpPacket {
    hardware_ty: [u8; 2], // must be [0, 1]
    protocol_ty: EtherType,
    hardware_len: u8, // must be 6
    protocol_len: u8, // must be 4
    op: [u8; 2],
    pub sender_eth_addr: EthernetAddress,
    pub sender_ipv4_addr: Ipv4Addr,
    pub target_eth_addr: EthernetAddress,
    pub target_ipv4_addr: Ipv4Addr,
}

impl From<ArpPacket> for [u8; 28] {
    fn from(value: ArpPacket) -> Self {
        let protocol_ty: [u8; 2] = value.protocol_ty.into();
        let sender_eth_addr: [u8; 6] = value.sender_eth_addr.into();
        let sender_ipv4_addr: [u8; 4] = value.sender_ipv4_addr.octets();
        let target_eth_addr: [u8; 6] = value.target_eth_addr.into();
        let target_ipv4_addr: [u8; 4] = value.target_ipv4_addr.octets();

        [
            value.hardware_ty[0],
            value.hardware_ty[1],
            protocol_ty[0],
            protocol_ty[1],
            value.hardware_len,
            value.protocol_len,
            value.op[0],
            value.op[1],
            sender_eth_addr[0],
            sender_eth_addr[1],
            sender_eth_addr[2],
            sender_eth_addr[3],
            sender_eth_addr[4],
            sender_eth_addr[5],
            sender_ipv4_addr[0],
            sender_ipv4_addr[1],
            sender_ipv4_addr[2],
            sender_ipv4_addr[3],
            target_eth_addr[0],
            target_eth_addr[1],
            target_eth_addr[2],
            target_eth_addr[3],
            target_eth_addr[4],
            target_eth_addr[5],
            target_ipv4_addr[0],
            target_ipv4_addr[1],
            target_ipv4_addr[2],
            target_ipv4_addr[3],
        ]
    }
}

impl From<[u8; 28]> for ArpPacket {
    fn from(value: [u8; 28]) -> Self {
        Self {
            hardware_ty: [value[0], value[1]],
            protocol_ty: [value[2], value[3]].into(),
            hardware_len: value[4],
            protocol_len: value[5],
            op: [value[6], value[7]],
            sender_eth_addr: [
                value[8], value[9], value[10], value[11], value[12], value[13],
            ]
            .into(),
            sender_ipv4_addr: [value[14], value[15], value[16], value[17]].into(),
            target_eth_addr: [
                value[18], value[19], value[20], value[21], value[22], value[23],
            ]
            .into(),
            target_ipv4_addr: [value[24], value[25], value[26], value[27]].into(),
        }
    }
}

impl ArpPacket {
    pub fn new_with(
        op: ArpOperation,
        sender_eth_addr: EthernetAddress,
        sender_ipv4_addr: Ipv4Addr,
        target_eth_addr: EthernetAddress,
        target_ipv4_addr: Ipv4Addr,
    ) -> Self {
        Self {
            hardware_ty: [0, 1],
            protocol_ty: EtherType::Ipv4,
            hardware_len: 6,
            protocol_len: 4,
            op: op.into(),
            sender_eth_addr,
            sender_ipv4_addr,
            target_eth_addr,
            target_ipv4_addr,
        }
    }

    pub fn op(&self) -> Result<ArpOperation> {
        ArpOperation::try_from(self.op)
    }
}
