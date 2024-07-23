use crate::sys;
use nex::packet::frame::{DatalinkLayer, IpLayer, TransportLayer};
use nex::packet::{arp, icmp};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Serialize, Deserialize)]
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
    //pub payload: Vec<u8>,
    /// Packet length.
    pub packet_len: usize,
    /// Packet arrival time. RFC3339 format.
    pub timestamp: String,
}

impl PacketFrame {
    pub fn new() -> Self {
        PacketFrame {
            capture_no: 0,
            if_index: 0,
            if_name: String::new(),
            datalink: None,
            ip: None,
            transport: None,
            //payload: Vec::new(),
            packet_len: 0,
            timestamp: String::new(),
        }
    }
    pub fn from_nex_frame(
        capture_no: usize,
        if_index: u32,
        if_name: String,
        frame: nex::packet::frame::Frame,
    ) -> PacketFrame {
        PacketFrame {
            capture_no: capture_no,
            if_index: if_index,
            if_name: if_name,
            datalink: frame.datalink,
            ip: frame.ip,
            transport: frame.transport,
            //payload: frame.payload,
            packet_len: frame.packet_len,
            timestamp: sys::get_sysdate(),
        }
    }
    pub fn get_time(&self) -> String {
        let datetime_vec: Vec<&str> = self.timestamp.split('T').collect::<Vec<&str>>();
        let timestamp: String = if datetime_vec.len() > 1 {
            datetime_vec[1].to_string()
        } else {
            self.timestamp.clone()
        };
        // Remove UTC offset that start from + or -
        let datetime_vec = timestamp.split('+').collect::<Vec<&str>>();
        let timestamp: String = if datetime_vec.len() > 1 {
            datetime_vec[0].to_string()
        } else if datetime_vec.len() > 1 {
            datetime_vec[0].to_string()
        } else {
            timestamp
        };
        timestamp
    }
    // Get most high level protocol
    pub fn get_protocol(&self) -> String {
        // Transport layer
        if let Some(transport) = &self.transport {
            if let Some(tcp) = &transport.tcp {
                return "TCP".to_string();
            }
            if let Some(udp) = &transport.udp {
                return "UDP".to_string();
            }
        }
        // IP layer
        if let Some(ip) = &self.ip {
            if let Some(icmp) = &ip.icmp {
                return "ICMP".to_string();
            }
            if let Some(icmpv6) = &ip.icmpv6 {
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
            if let Some(arp) = &datalink.arp {
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
        if let Some(datalink) = &self.datalink {
            if let Some(ethernet) = &datalink.ethernet {
                return ethernet.source.to_string();
            }
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
        if let Some(datalink) = &self.datalink {
            if let Some(ethernet) = &datalink.ethernet {
                return ethernet.destination.to_string();
            }
        }
        "Unknown".to_string()
    }
    pub fn get_src_port(&self) -> String {
        if let Some(transport) = &self.transport {
            if let Some(tcp) = &transport.tcp {
                return tcp.source.to_string();
            }
            if let Some(udp) = &transport.udp {
                return udp.source.to_string();
            }
        }
        String::new()
    }
    pub fn get_dst_port(&self) -> String {
        if let Some(transport) = &self.transport {
            if let Some(tcp) = &transport.tcp {
                return tcp.destination.to_string();
            }
            if let Some(udp) = &transport.udp {
                return udp.destination.to_string();
            }
        }
        String::new()
    }
}

pub struct PacketStorage {
    storage: Arc<Mutex<VecDeque<PacketFrame>>>,
    capture_counter: Arc<AtomicUsize>,
    max_capacity: usize,
}

impl PacketStorage {
    pub fn new() -> Self {
        PacketStorage {
            storage: Arc::new(Mutex::new(VecDeque::new())),
            capture_counter: Arc::new(AtomicUsize::new(1)),
            max_capacity: u8::MAX as usize,
        }
    }

    pub fn generate_capture_no(&self) -> usize {
        self.capture_counter.fetch_add(1, Ordering::SeqCst)
    }

    pub fn add_packet(&self, packet: PacketFrame) {
        match self.storage.try_lock() {
            Ok(mut storage) => {
                // If the storage is full, remove the oldest packet
                if storage.len() == self.max_capacity {
                    storage.pop_front();
                }
                storage.push_back(packet);
            }
            Err(_) => {
                // TODO: Log error or return error
            }
        }
    }

    pub fn get_packets(&self) -> Vec<PacketFrame> {
        match self.storage.lock() {
            Ok(storage) => storage.iter().cloned().collect(),
            Err(_) => {
                // TODO: Log error or return error
                Vec::new()
            },
        }
    }
}
