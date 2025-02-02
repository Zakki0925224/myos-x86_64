use crate::{
    error::{Error, Result},
    util::mutex::Mutex,
};
use arp::{ArpOperation, ArpPacket, ArpTable};
use core::net::Ipv4Addr;
use eth::{EthernetAddress, EthernetPayload};
use log::info;

pub mod arp;
pub mod eth;

static mut NETWORK_MAN: Mutex<NetworkManager> =
    Mutex::new(NetworkManager::new(Ipv4Addr::new(10, 0, 2, 15)));

struct NetworkManager {
    my_ipv4_addr: Ipv4Addr,
    my_mac_addr: Option<EthernetAddress>,
    arp_table: Option<ArpTable>,
}

impl NetworkManager {
    pub const fn new(ipv4_addr: Ipv4Addr) -> Self {
        Self {
            my_ipv4_addr: ipv4_addr,
            my_mac_addr: None,
            arp_table: None,
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

    fn arp_table(&mut self) -> &mut ArpTable {
        if self.arp_table.is_none() {
            self.arp_table = Some(ArpTable::new());
        }

        self.arp_table.as_mut().unwrap()
    }

    fn receive_arp_packet(&mut self, packet: ArpPacket) -> Result<Option<ArpPacket>> {
        info!("net: Received ARP packet: {:?}", packet);

        let arp_op = packet.op()?;
        let sender_ipv4_addr = packet.sender_ipv4_addr;
        let sender_mac_addr = packet.sender_eth_addr;
        let target_ipv4_addr = packet.target_ipv4_addr;

        match arp_op {
            ArpOperation::Request => {
                let arp_table = self.arp_table();
                arp_table.insert(sender_ipv4_addr, sender_mac_addr);
                info!("net: ARP table updated: {:?}", arp_table);

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

    fn receive_eth_payload(&mut self, payload: EthernetPayload) -> Result<Option<EthernetPayload>> {
        let mut replay_payload = None;

        match payload {
            EthernetPayload::Arp(arp_packet) => {
                if let Some(reply_arp_packet) = self.receive_arp_packet(arp_packet)? {
                    replay_payload = Some(EthernetPayload::Arp(reply_arp_packet));
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
