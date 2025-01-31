use super::arp;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EtherType {
    IPv4,
    IPv6,
    ARP,
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
                0x0806 => EtherType::ARP,
                _ => EtherType::Other(value),
            }
        }
    }
}

#[derive(Debug)]
pub enum EthernetPayload {
    ARP(arp::ArpPacket),
    None,
}

pub struct EthernetPacket<'a> {
    data: &'a [u8],
}

impl<'a> EthernetPacket<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    pub fn raw(&self) -> &'a [u8] {
        self.data
    }

    pub fn dst_mac(&self) -> EthernetAddress {
        let mac = &self.data[0..6];
        [mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]].into()
    }

    pub fn src_mac(&self) -> EthernetAddress {
        let mac = &self.data[6..12];
        [mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]].into()
    }

    pub fn ether_type(&self) -> EtherType {
        [self.data[12], self.data[13]].into()
    }

    pub fn payload(&self) -> EthernetPayload {
        match self.ether_type() {
            EtherType::ARP => {
                let arp_packet = arp::ArpPacket::new(&self.data[14..]);
                EthernetPayload::ARP(arp_packet)
            }
            _ => EthernetPayload::None,
        }
    }
}
