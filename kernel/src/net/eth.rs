use super::arp;
use alloc::vec::Vec;
use core::fmt::Debug;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct EthernetAddress([u8; 6]);

impl Debug for EthernetAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mac = self.0;

        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
        )
    }
}

impl From<[u8; 6]> for EthernetAddress {
    fn from(mac: [u8; 6]) -> Self {
        Self(mac)
    }
}

impl Into<[u8; 6]> for EthernetAddress {
    fn into(self) -> [u8; 6] {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EtherType {
    IPv4,
    IPv6,
    Arp,
    Other(u16),
    PayloadLength(u16),
}

impl From<[u8; 2]> for EtherType {
    fn from(data: [u8; 2]) -> Self {
        let value = u16::from_be_bytes(data);

        if value <= 0x05dc {
            EtherType::PayloadLength(value)
        } else {
            match value {
                0x0800 => EtherType::IPv4,
                0x86dd => EtherType::IPv6,
                0x0806 => EtherType::Arp,
                _ => EtherType::Other(value),
            }
        }
    }
}

impl Into<[u8; 2]> for EtherType {
    fn into(self) -> [u8; 2] {
        match self {
            EtherType::IPv4 => [0x08, 0x00],
            EtherType::IPv6 => [0x86, 0xdd],
            EtherType::Arp => [0x08, 0x06],
            EtherType::Other(value) => value.to_be_bytes(),
            EtherType::PayloadLength(value) => value.to_be_bytes(),
        }
    }
}

#[derive(Debug)]
pub enum EthernetPayload {
    Arp(arp::ArpPacket),
    None,
}

impl EthernetPayload {
    pub fn to_vec(&self) -> Vec<u8> {
        match self {
            EthernetPayload::Arp(packet) => packet.raw().to_vec(),
            EthernetPayload::None => Vec::new(),
        }
    }
}

pub struct EthernetFrame<'a> {
    pub dst_mac_addr: EthernetAddress,
    pub src_mac_addr: EthernetAddress,
    pub ether_type: EtherType,
    payload: &'a [u8],
    // fcs: u32,
}

impl<'a> Debug for EthernetFrame<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "EthernetFrame {{ dst: {:?}, src: {:?}, ether_type: {:?}, payload: {:?} }}",
            self.dst_mac_addr,
            self.src_mac_addr,
            self.ether_type,
            self.payload()
        )
    }
}

impl<'a> EthernetFrame<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        let dst_mac = &data[0..6];
        let src_mac = &data[6..12];
        let ether_type = [data[12], data[13]].into();
        let payload = &data[14..];

        Self {
            dst_mac_addr: [
                dst_mac[0], dst_mac[1], dst_mac[2], dst_mac[3], dst_mac[4], dst_mac[5],
            ]
            .into(),
            src_mac_addr: [
                src_mac[0], src_mac[1], src_mac[2], src_mac[3], src_mac[4], src_mac[5],
            ]
            .into(),
            ether_type,
            payload,
        }
    }

    pub fn new_with(
        dst_mac_addr: EthernetAddress,
        src_mac_addr: EthernetAddress,
        ether_type: EtherType,
        payload: &'a [u8],
    ) -> Self {
        Self {
            dst_mac_addr,
            src_mac_addr,
            ether_type,
            payload,
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut vec = Vec::new();
        let dst_mac_addr: [u8; 6] = self.dst_mac_addr.into();
        let src_mac_addr: [u8; 6] = self.src_mac_addr.into();
        let ether_type: [u8; 2] = self.ether_type.into();
        let mut payload = self.payload().to_vec();

        if payload.len() < 46 {
            payload.resize(46, 0);
        }

        vec.extend_from_slice(&dst_mac_addr);
        vec.extend_from_slice(&src_mac_addr);
        vec.extend_from_slice(&ether_type);
        vec.extend_from_slice(&payload);

        // padding
        if vec.len() < 64 {
            vec.resize(64, 0);
        }

        vec
    }

    pub fn payload(&self) -> EthernetPayload {
        match self.ether_type {
            EtherType::Arp => {
                let arp_packet = arp::ArpPacket::new(&self.payload);
                EthernetPayload::Arp(arp_packet)
            }
            _ => EthernetPayload::None,
        }
    }
}
