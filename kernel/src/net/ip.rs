use core::net::Ipv4Addr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Protocol {
    Icmp = 1,
    Igmp = 2,
    Tcp = 6,
    Udp = 17,
    Gre = 47,
    Esp = 50,
    Eigrp = 88,
    Ospf = 89,
    Vrrp = 112,
    Other(u8),
}

impl From<u8> for Protocol {
    fn from(data: u8) -> Self {
        match data {
            1 => Protocol::Icmp,
            2 => Protocol::Igmp,
            6 => Protocol::Tcp,
            17 => Protocol::Udp,
            47 => Protocol::Gre,
            50 => Protocol::Esp,
            88 => Protocol::Eigrp,
            89 => Protocol::Ospf,
            112 => Protocol::Vrrp,
            _ => Protocol::Other(data),
        }
    }
}

#[derive(Debug)]
pub struct Ipv4Packet {
    version_ihl: u8,
    dscp_ecn: u8,
    len: u16,
    id: u16,
    flags: u16,
    ttl: u8,
    protocol: Protocol,
    checksum: u16,
    src_addr: Ipv4Addr,
    dst_addr: Ipv4Addr,
}

impl Ipv4Packet {
    pub fn new(data: &[u8]) -> Self {
        Self {
            version_ihl: data[0],
            dscp_ecn: data[1],
            len: u16::from_be_bytes([data[2], data[3]]),
            id: u16::from_be_bytes([data[4], data[5]]),
            flags: u16::from_be_bytes([data[6], data[7]]),
            ttl: data[8],
            protocol: data[9].into(),
            checksum: u16::from_be_bytes([data[10], data[11]]),
            src_addr: [data[12], data[13], data[14], data[15]].into(),
            dst_addr: [data[16], data[17], data[18], data[19]].into(),
        }
    }
}
