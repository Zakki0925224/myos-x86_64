use crate::{
    error::{Error, Result},
    util::mutex::Mutex,
};
use alloc::collections::btree_map::BTreeMap;
use arp::{ArpOperation, ArpPacket};
use core::net::Ipv4Addr;
use eth::{EthernetAddress, EthernetPayload};
use icmp::{IcmpPacket, IcmpType};
use ip::{Ipv4Packet, Ipv4Payload};
use log::info;
use tcp::{TcpPacket, TcpSocket};

pub mod arp;
pub mod eth;
pub mod icmp;
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

    fn receive_icmp_packet(&mut self, packet: IcmpPacket) -> Result<Option<IcmpPacket>> {
        info!("net: Received ICMP packet");

        let ty = packet.ty;

        match ty {
            IcmpType::EchoRequest => {
                let mut reply_packet = packet.clone();
                reply_packet.ty = IcmpType::EchoReply;
                reply_packet.calc_checksum();
                return Ok(Some(reply_packet));
            }
            _ => (),
        }

        Ok(None)
    }

    fn receive_tcp_packet(&mut self, packet: TcpPacket) -> Result<Option<TcpPacket>> {
        info!("net: Received TCP packet");

        let dst_port = packet.dst_port;
        let socket = self.tcp_socket_mut(dst_port);
        info!("net: TCP socket({}): {:?}", dst_port, socket);

        Ok(None)
    }

    fn receive_arp_packet(&mut self, packet: ArpPacket) -> Result<Option<ArpPacket>> {
        info!("net: Received ARP packet");

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

                Ok(Some(reply_packet))
            }
            ArpOperation::Reply => {
                unimplemented!()
            }
        }
    }

    fn receive_ipv4_packet(&mut self, packet: Ipv4Packet) -> Result<Option<Ipv4Packet>> {
        info!("net: Received IPv4 packet");

        packet.validate()?;

        if packet.dst_addr != self.my_ipv4_addr {
            return Ok(None);
        }

        let mut reply_payload = None;
        match packet.payload()? {
            Ipv4Payload::Icmp(icmp_packet) => {
                if let Some(reply_icmp_packet) = self.receive_icmp_packet(icmp_packet)? {
                    reply_payload = Some(Ipv4Payload::Icmp(reply_icmp_packet));
                }
            }
            Ipv4Payload::Tcp(tcp_packet) => {
                self.receive_tcp_packet(tcp_packet)?;
            }
        }

        let mut reply_packet = None;
        if let Some(reply_payload) = reply_payload {
            let mut ipv4_packet = Ipv4Packet::new_with(
                packet.version_ihl,
                packet.dscp_ecn,
                packet.id,
                packet.flags,
                packet.protocol,
                packet.dst_addr,
                packet.src_addr,
                reply_payload,
            );
            ipv4_packet.calc_checksum();
            reply_packet = Some(ipv4_packet);
        }

        Ok(reply_packet)
    }

    fn receive_eth_payload(&mut self, payload: EthernetPayload) -> Result<Option<EthernetPayload>> {
        info!("net: Received Ethernet payload");

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
            EthernetPayload::None => (),
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
