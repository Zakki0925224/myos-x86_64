use crate::error::Error;
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

#[derive(Debug)]
pub struct UdpSocket {
    buf: Vec<u8>,
}

impl UdpSocket {
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }

    pub fn receive(&mut self, data: &[u8]) {
        self.buf.extend_from_slice(data);
    }

    pub fn buf_to_string_utf8_lossy(&self) -> String {
        String::from_utf8_lossy(&self.buf).to_string()
    }
}

#[derive(Debug, Clone)]
pub struct UdpPacket {
    src_port: u16,
    pub dst_port: u16,
    len: u16,
    checksum: u16,
    pub data: Vec<u8>,
}

impl TryFrom<&[u8]> for UdpPacket {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 8 {
            return Err(Error::Failed("Invalid data length"));
        }

        let src_port = u16::from_be_bytes([value[0], value[1]]);
        let dst_port = u16::from_be_bytes([value[2], value[3]]);
        let len = u16::from_be_bytes([value[4], value[5]]);
        let checksum = u16::from_be_bytes([value[6], value[7]]);
        let data = value[8..(len as usize)].to_vec();

        Ok(Self {
            src_port,
            dst_port,
            len,
            checksum,
            data,
        })
    }
}

impl UdpPacket {
    pub fn to_vec(&self) -> Vec<u8> {
        let mut vec = Vec::new();
        vec.extend_from_slice(&self.src_port.to_be_bytes());
        vec.extend_from_slice(&self.dst_port.to_be_bytes());
        vec.extend_from_slice(&self.len.to_be_bytes());
        vec.extend_from_slice(&self.checksum.to_be_bytes());
        vec.extend_from_slice(&self.data);
        vec
    }
}
