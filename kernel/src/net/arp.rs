use super::eth::{EtherType, EthernetAddress};
use crate::error::{Error, Result};
use alloc::collections::BTreeMap;
use core::net::Ipv4Addr;

pub type ArpTable = BTreeMap<Ipv4Addr, EthernetAddress>;

pub enum ArpOperation {
    Request,
    Reply,
}

impl Into<[u8; 2]> for ArpOperation {
    fn into(self) -> [u8; 2] {
        match self {
            ArpOperation::Request => [0, 1],
            ArpOperation::Reply => [0, 2],
        }
    }
}

#[derive(Debug)]
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

impl ArpPacket {
    pub fn new(data: &[u8]) -> Self {
        Self {
            hardware_ty: [data[0], data[1]],
            protocol_ty: [data[2], data[3]].into(),
            hardware_len: data[4],
            protocol_len: data[5],
            op: [data[6], data[7]],
            sender_eth_addr: [data[8], data[9], data[10], data[11], data[12], data[13]].into(),
            sender_ipv4_addr: [data[14], data[15], data[16], data[17]].into(),
            target_eth_addr: [data[18], data[19], data[20], data[21], data[22], data[23]].into(),
            target_ipv4_addr: [data[24], data[25], data[26], data[27]].into(),
        }
    }

    pub fn new_with(
        op: ArpOperation,
        sender_eth_addr: EthernetAddress,
        sender_ipv4_addr: Ipv4Addr,
        target_eth_addr: EthernetAddress,
        target_ipv4_addr: Ipv4Addr,
    ) -> Self {
        Self {
            hardware_ty: [0, 1],
            protocol_ty: EtherType::IPv4,
            hardware_len: 6,
            protocol_len: 4,
            op: op.into(),
            sender_eth_addr,
            sender_ipv4_addr,
            target_eth_addr,
            target_ipv4_addr,
        }
    }

    pub fn raw(&self) -> [u8; 28] {
        let protocol_ty: [u8; 2] = self.protocol_ty.into();
        let sender_eth_addr: [u8; 6] = self.sender_eth_addr.into();
        let sender_ipv4_addr: [u8; 4] = self.sender_ipv4_addr.octets();
        let target_eth_addr: [u8; 6] = self.target_eth_addr.into();
        let target_ipv4_addr: [u8; 4] = self.target_ipv4_addr.octets();

        [
            self.hardware_ty[0],
            self.hardware_ty[1],
            protocol_ty[0],
            protocol_ty[1],
            self.hardware_len,
            self.protocol_len,
            self.op[0],
            self.op[1],
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

    pub fn op(&self) -> Result<ArpOperation> {
        match self.op {
            [0, 1] => Ok(ArpOperation::Request),
            [0, 2] => Ok(ArpOperation::Reply),
            _ => Err(Error::Failed("Invalid ARP operation")),
        }
    }
}
