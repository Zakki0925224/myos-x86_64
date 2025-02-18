use crate::{
    error::{Error, Result},
    util::mutex::Mutex,
};
use alloc::collections::btree_map::BTreeMap;
use arp::{ArpOperation, ArpPacket};
use core::net::Ipv4Addr;
use eth::{EthernetAddress, EthernetPayload};
use ip::{Ipv4Packet, Ipv4Payload};
use log::info;
use tcp::{TcpPacket, TcpSocket};

pub mod arp;
pub mod eth;
pub mod ip;
pub mod tcp;

type ArpTable = BTreeMap<Ipv4Addr, EthernetAddress>;
type TcpSocketTable = BTreeMap<u16, TcpSocket>;

static mut NETWORK_MAN: Mutex<NetworkManager> =
    Mutex::new(NetworkManager::new(Ipv4Addr::new(192, 168, 100, 2)));

struct NetworkManager {
    my_ipv4_addr: Ipv4Addr,
    my_mac_addr: Option<EthernetAddress>,
    arp_table: ArpTable,
    tcp_socket_table: TcpSocketTable,
}

impl NetworkManager {
    const fn new(ipv4_addr: Ipv4Addr) -> Self {
        Self {
            my_ipv4_addr: ipv4_addr,
            my_mac_addr: None,
            arp_table: ArpTable::new(),
            tcp_socket_table: TcpSocketTable::new(),
        }
    }

    fn set_my_mac_addr(&mut self, mac_addr: EthernetAddress) {
        self.my_mac_addr = Some(mac_addr);

        info!("net: MAC address set to {:?}", mac_addr);
    }

    fn my_mac_addr(&self) -> Result<EthernetAddress> {
        self.my_mac_addr
            .ok_or(Error::Failed("MAC address is not set"))
    }

    fn tcp_socket_mut(&mut self, port: u16) -> &mut TcpSocket {
        self.tcp_socket_table
            .entry(port)
            .or_insert_with(TcpSocket::new)
    }

    fn receive_tcp_packet(&mut self, packet: TcpPacket) -> Result<Option<TcpPacket>> {
        let dst_port = packet.dst_port;
        let socket = self.tcp_socket_mut(dst_port);
        info!("net: TCP socket({}): {:?}", dst_port, socket);

        Ok(None)
    }

    fn receive_arp_packet(&mut self, packet: ArpPacket) -> Result<Option<ArpPacket>> {
        info!("net: Received ARP packet: {:?}", packet);

        let arp_op = packet.op()?;
        let sender_ipv4_addr = packet.sender_ipv4_addr;
        let sender_mac_addr = packet.sender_eth_addr;
        let target_ipv4_addr = packet.target_ipv4_addr;

        match arp_op {
            ArpOperation::Request => {
                self.arp_table.insert(sender_ipv4_addr, sender_mac_addr);
                info!("net: ARP table updated: {:?}", self.arp_table);

                if target_ipv4_addr != self.my_ipv4_addr {
                    return Ok(None);
                }

                let reply_packet = ArpPacket::new_with(
                    ArpOperation::Reply,
                    self.my_mac_addr()?,
                    self.my_ipv4_addr,
                    sender_mac_addr,
                    sender_ipv4_addr,
                );
                info!("net: Generated ARP reply packet: {:?}", reply_packet);

                Ok(Some(reply_packet))
            }
            ArpOperation::Reply => {
                unimplemented!()
            }
        }
    }

    fn receive_ipv4_packet(&mut self, packet: Ipv4Packet) -> Result<Option<Ipv4Packet>> {
        info!("net: Received IPv4 packet: {:?}", packet);
        packet.validate()?;

        if packet.dst_addr != self.my_ipv4_addr {
            return Ok(None);
        }

        match packet.payload()? {
            Ipv4Payload::Tcp(tcp_packet) => {
                self.receive_tcp_packet(tcp_packet)?;
            }
        }

        Ok(None)
    }

    fn receive_eth_payload(&mut self, payload: EthernetPayload) -> Result<Option<EthernetPayload>> {
        let mut replay_payload = None;

        match payload {
            EthernetPayload::Arp(arp_packet) => {
                if let Some(reply_arp_packet) = self.receive_arp_packet(arp_packet)? {
                    replay_payload = Some(EthernetPayload::Arp(reply_arp_packet));
                }
            }
            EthernetPayload::Ipv4(ipv4_packet) => {
                if let Some(reply_ipv4_packet) = self.receive_ipv4_packet(ipv4_packet)? {
                    replay_payload = Some(EthernetPayload::Ipv4(reply_ipv4_packet));
                }
            }
            EthernetPayload::None => {
                info!("net: None payload");
            }
        }

        Ok(replay_payload)
    }
}

pub fn set_my_mac_addr(mac_addr: EthernetAddress) -> Result<()> {
    unsafe { NETWORK_MAN.try_lock() }?.set_my_mac_addr(mac_addr);
    Ok(())
}

pub fn my_mac_addr() -> Result<EthernetAddress> {
    unsafe { NETWORK_MAN.try_lock() }?.my_mac_addr()
}

pub fn receive_eth_payload(payload: EthernetPayload) -> Result<Option<EthernetPayload>> {
    unsafe { NETWORK_MAN.try_lock() }?.receive_eth_payload(payload)
}
