use crate::sys;
use bytes::Bytes;
use nex::packet::ethernet::EtherType;
use nex::packet::frame::{DatalinkLayer, IpLayer, TransportLayer};
use nex::packet::ip::IpNextProtocol;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PacketFrame {
    /// Capture number.
    pub capture_no: usize,
    /// interface index
    pub if_index: u32,
    /// interface name.
    pub if_name: String,
    /// The datalink layer.
    pub datalink: Option<DatalinkLayer>,
    /// The IP layer.
    pub ip: Option<IpLayer>,
    /// The transport layer.
    pub transport: Option<TransportLayer>,
    /// Rest of the packet that could not be parsed as a header. (Usually payload)
    pub payload: Bytes,
    /// Packet length.
    pub packet_len: usize,
    /// Packet arrival time. RFC3339 format.
    pub timestamp: String,
}

impl PacketFrame {
    pub fn from_nex_frame(
        capture_no: usize,
        if_index: u32,
        if_name: String,
        frame: nex::packet::frame::Frame,
    ) -> PacketFrame {
        PacketFrame {
            capture_no,
            if_index,
            if_name,
            datalink: frame.datalink,
            ip: frame.ip,
            transport: frame.transport,
            payload: frame.payload,
            packet_len: frame.packet_len,
            timestamp: sys::get_sysdate(),
        }
    }
    pub fn get_time(&self) -> String {
        chrono::DateTime::parse_from_rfc3339(&self.timestamp)
            .map(|timestamp| timestamp.format("%H:%M:%S%.3f").to_string())
            .unwrap_or_else(|_| self.timestamp.clone())
    }
    // Get most high level protocol
    pub fn get_protocol(&self) -> String {
        // Transport layer
        if let Some(transport) = &self.transport {
            if let Some(_tcp) = &transport.tcp {
                return "TCP".to_string();
            }
            if let Some(_udp) = &transport.udp {
                return "UDP".to_string();
            }
        }
        // IP layer
        if let Some(ip) = &self.ip {
            if let Some(_icmp) = &ip.icmp {
                return "ICMP".to_string();
            }
            if let Some(_icmpv6) = &ip.icmpv6 {
                return "ICMPv6".to_string();
            }
            if let Some(ipv4) = &ip.ipv4 {
                return ipv4.next_level_protocol.as_str().to_string();
            }
            if let Some(ipv6) = &ip.ipv6 {
                return ipv6.next_header.as_str().to_string();
            }
        }
        // Datalink layer
        if let Some(datalink) = &self.datalink {
            if let Some(_arp) = &datalink.arp {
                return "ARP".to_string();
            }
            if let Some(ethernet) = &datalink.ethernet {
                return ethernet.ethertype.name().to_string();
            }
        }
        "Unknown".to_string()
    }
    pub fn get_src_addr(&self) -> String {
        if let Some(ip) = &self.ip {
            if let Some(ipv4) = &ip.ipv4 {
                return ipv4.source.to_string();
            }
            if let Some(ipv6) = &ip.ipv6 {
                return ipv6.source.to_string();
            }
        }
        if let Some(datalink) = &self.datalink
            && let Some(ethernet) = &datalink.ethernet
        {
            return ethernet.source.to_string();
        }
        "Unknown".to_string()
    }
    pub fn get_dst_addr(&self) -> String {
        if let Some(ip) = &self.ip {
            if let Some(ipv4) = &ip.ipv4 {
                return ipv4.destination.to_string();
            }
            if let Some(ipv6) = &ip.ipv6 {
                return ipv6.destination.to_string();
            }
        }
        if let Some(datalink) = &self.datalink
            && let Some(ethernet) = &datalink.ethernet
        {
            return ethernet.destination.to_string();
        }
        "Unknown".to_string()
    }
}

pub struct PacketStorage {
    storage: Arc<RwLock<VecDeque<PacketFrame>>>,
    capture_counter: Arc<AtomicUsize>,
    max_capacity: usize,
}

impl PacketStorage {
    pub fn with_capacity(capacity: usize) -> Self {
        PacketStorage {
            storage: Arc::new(RwLock::new(VecDeque::new())),
            capture_counter: Arc::new(AtomicUsize::new(1)),
            max_capacity: capacity.max(1),
        }
    }

    pub fn generate_capture_no(&self) -> usize {
        self.capture_counter.fetch_add(1, Ordering::Relaxed)
    }

    pub fn add_packet(&self, packet: PacketFrame) {
        match self.storage.write() {
            Ok(mut storage) => {
                // If the storage is full, remove the oldest packet
                while storage.len() >= self.max_capacity {
                    storage.pop_front();
                }
                storage.push_back(packet);
            }
            Err(e) => {
                tracing::error!("failed to lock packet storage for write: {}", e);
                let mut storage = e.into_inner();
                while storage.len() >= self.max_capacity {
                    storage.pop_front();
                }
                storage.push_back(packet);
            }
        }
    }

    pub fn get_packets(&self) -> Vec<PacketFrame> {
        match self.storage.read() {
            Ok(storage) => storage.iter().cloned().collect(),
            Err(e) => {
                tracing::error!("failed to lock packet storage for read: {}", e);
                e.into_inner().iter().cloned().collect()
            }
        }
    }
}

pub fn get_ethertype_from_str(ethertype_name: &str) -> Option<EtherType> {
    let name = ethertype_name.to_lowercase();
    // Currently, EtherType not support from_str, so we need to match it manually
    match name.as_str() {
        "arp" => Some(EtherType::Arp),
        "rarp" => Some(EtherType::Rarp),
        "aarp" => Some(EtherType::Aarp),
        "ipv4" => Some(EtherType::Ipv4),
        "ipv6" => Some(EtherType::Ipv6),
        "vlan" => Some(EtherType::Vlan),
        "mpls" => Some(EtherType::Mpls),
        "wakeonlan" => Some(EtherType::WakeOnLan),
        "rldp" => Some(EtherType::Rldp),
        "lldp" => Some(EtherType::Lldp),
        _ => None,
    }
}

pub fn get_ip_next_protocol_from_str(protocol_name: &str) -> Option<IpNextProtocol> {
    let name = protocol_name.to_lowercase();
    // Currently, IpNextLevelProtocol not support from_str, so we need to match it manually
    match name.as_str() {
        "icmp" => Some(IpNextProtocol::Icmp),
        "icmpv6" => Some(IpNextProtocol::Icmpv6),
        "tcp" => Some(IpNextProtocol::Tcp),
        "udp" => Some(IpNextProtocol::Udp),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn packet(capture_no: usize) -> PacketFrame {
        PacketFrame {
            capture_no,
            ..Default::default()
        }
    }

    #[test]
    fn storage_never_accepts_an_unbounded_zero_capacity() {
        let storage = PacketStorage::with_capacity(0);
        storage.add_packet(packet(1));
        storage.add_packet(packet(2));
        let packets = storage.get_packets();
        assert_eq!(packets.len(), 1);
        assert_eq!(packets[0].capture_no, 2);
    }

    #[test]
    fn timestamp_display_handles_negative_offsets() {
        let packet = PacketFrame {
            timestamp: "2026-08-12T07:08:09.123-07:00".to_string(),
            ..Default::default()
        };
        assert_eq!(packet.get_time(), "07:08:09.123");
    }
}
