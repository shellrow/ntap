use crate::sys;
use nex::packet::frame::{DatalinkLayer, IpLayer, TransportLayer};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
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
}

pub struct PacketStorage {
    storage: Arc<Mutex<VecDeque<PacketFrame>>>,
    max_capacity: usize,
}

impl PacketStorage {
    pub fn new() -> Self {
        PacketStorage {
            storage: Arc::new(Mutex::new(VecDeque::new())),
            max_capacity: u8::MAX as usize,
        }
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
