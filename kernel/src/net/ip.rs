use super::{icmp::IcmpPacket, tcp::TcpPacket};
use crate::error::{Error, Result};
use alloc::vec::Vec;
use core::{fmt::Debug, net::Ipv4Addr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    Icmp,
    Tcp,
    Udp,
    Other(u8),
}

impl From<Protocol> for u8 {
    fn from(proto: Protocol) -> Self {
        match proto {
            Protocol::Icmp => 1,
            Protocol::Tcp => 6,
            Protocol::Udp => 17,
            Protocol::Other(x) => x,
        }
    }
}

impl From<u8> for Protocol {
    fn from(data: u8) -> Self {
        match data {
            1 => Protocol::Icmp,
            6 => Protocol::Tcp,
            17 => Protocol::Udp,
            _ => Protocol::Other(data),
        }
    }
}

#[derive(Debug)]
pub enum Ipv4Payload {
    Icmp(IcmpPacket),
    Tcp(TcpPacket),
}

#[derive(Clone)]
pub struct Ipv4Packet {
    version_ihl: u8,
    dscp_ecn: u8,
    len: u16,
    id: u16,
    flags: u16,
    ttl: u8,
    pub protocol: Protocol,
    checksum: u16,
    pub src_addr: Ipv4Addr,
    pub dst_addr: Ipv4Addr,
    data: Vec<u8>, // options, padding, data
}

impl Debug for Ipv4Packet {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Ipv4Packet")
            .field("version_ihl", &self.version_ihl)
            .field("dscp_ecn", &self.dscp_ecn)
            .field("len", &self.len)
            .field("id", &self.id)
            .field("flags", &self.flags)
            .field("ttl", &self.ttl)
            .field("protocol", &self.protocol)
            .field("checksum", &self.checksum)
            .field("src_addr", &self.src_addr)
            .field("dst_addr", &self.dst_addr)
            .field("payload", &self.payload())
            .finish()
    }
}

impl TryFrom<&[u8]> for Ipv4Packet {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self> {
        if value.len() < 20 {
            return Err("Invalid data length".into());
        }

        let version_ihl = value[0];
        let dscp_ecn = value[1];
        let len = u16::from_be_bytes([value[2], value[3]]);
        let id = u16::from_be_bytes([value[4], value[5]]);
        let flags = u16::from_be_bytes([value[6], value[7]]);
        let ttl = value[8];
        let protocol = value[9].into();
        let checksum = u16::from_be_bytes([value[10], value[11]]);
        let src_addr = Ipv4Addr::new(value[12], value[13], value[14], value[15]);
        let dst_addr = Ipv4Addr::new(value[16], value[17], value[18], value[19]);
        let data = value[20..].to_vec();

        Ok(Self {
            version_ihl,
            dscp_ecn,
            len,
            id,
            flags,
            ttl,
            protocol,
            checksum,
            src_addr,
            dst_addr,
            data,
        })
    }
}

impl Ipv4Packet {
    pub fn validate(&self) -> Result<()> {
        let version = self.version_ihl >> 4;

        if version != 4 {
            return Err("Invalid version".into());
        }

        if self.ttl == 0 {
            return Err("TTL is 0".into());
        }

        Ok(())
    }

    pub fn payload(&self) -> Result<Ipv4Payload> {
        let data_slice = self.data.as_slice();

        let payload = match self.protocol {
            Protocol::Icmp => Ipv4Payload::Icmp(IcmpPacket::try_from(data_slice)?),
            Protocol::Tcp => Ipv4Payload::Tcp(TcpPacket::try_from(data_slice)?),
            _ => return Err("Unsupported protocol".into()),
        };

        Ok(payload)
    }
}
