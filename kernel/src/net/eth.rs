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

impl EthernetAddress {
    const ETH_ADDR_BROADCAST: Self = Self::new([0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);

    pub const fn new(mac: [u8; 6]) -> Self {
        Self(mac)
    }

    pub fn raw(&self) -> [u8; 6] {
        self.0
    }

    pub fn is_broadcast(&self) -> bool {
        *self == Self::ETH_ADDR_BROADCAST
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

impl EtherType {
    pub const fn new(value: u16) -> Self {
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
        EthernetAddress::new([mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]])
    }

    pub fn src_mac(&self) -> EthernetAddress {
        let mac = &self.data[6..12];
        EthernetAddress::new([mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]])
    }

    pub fn ether_type(&self) -> EtherType {
        EtherType::new(u16::from_be_bytes([self.data[12], self.data[13]]))
    }
}
