use crate::error::{Error, Result};
use alloc::vec::Vec;

#[derive(Debug)]
pub enum TcpSocketState {
    Listen,
    SynSent,
    SynReceived,
    Established,
    FinWait1,
    FinWait2,
    CloseWait,
    Closing,
    LastAck,
    TimeWait,
    Closed,
}

#[derive(Debug)]
pub struct TcpSocket {
    state: TcpSocketState,
}

impl TcpSocket {
    pub fn new() -> Self {
        Self {
            state: TcpSocketState::Closed,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TcpPacket {
    src_port: u16,
    pub dst_port: u16,
    seq_num: u32,
    ack_num: u32,
    flags: u16,
    window_size: u16,
    checksum: u16,
    urgent_ptr: u16,
    options: Vec<u8>,
}

impl TryFrom<&[u8]> for TcpPacket {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self> {
        if value.len() < 20 {
            return Err(Error::Failed("Invalid data length"));
        }

        let src_port = u16::from_be_bytes([value[0], value[1]]);
        let dst_port = u16::from_be_bytes([value[2], value[3]]);
        let seq_num = u32::from_be_bytes([value[4], value[5], value[6], value[7]]);
        let ack_num = u32::from_be_bytes([value[8], value[9], value[10], value[11]]);
        let flags = u16::from_be_bytes([value[12], value[13]]);
        let window_size = u16::from_be_bytes([value[14], value[15]]);
        let checksum = u16::from_be_bytes([value[16], value[17]]);
        let urgent_ptr = u16::from_be_bytes([value[18], value[19]]);
        let options = value[20..].to_vec();

        Ok(Self {
            src_port,
            dst_port,
            seq_num,
            ack_num,
            flags,
            window_size,
            checksum,
            urgent_ptr,
            options,
        })
    }
}
