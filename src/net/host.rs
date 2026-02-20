use super::traffic::{TrafficDisplayInfo, TrafficInfo};
use serde::{Deserialize, Serialize};
use std::{net::IpAddr, time::Duration};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RemoteHostInfo {
    pub mac_addr: String,
    pub ip_addr: IpAddr,
    pub hostname: String,
    pub traffic_info: TrafficInfo,
}

impl RemoteHostInfo {
    pub fn new(mac_addr: String, ip_addr: IpAddr, hostname: String) -> Self {
        RemoteHostInfo {
            mac_addr,
            ip_addr,
            hostname,
            traffic_info: TrafficInfo::new(),
        }
    }
    pub fn merge(&mut self, other: &RemoteHostInfo, duration: Duration) {
        // Update traffic_info
        self.traffic_info
            .update_bytes_per_sec(&other.traffic_info, duration);
        self.traffic_info.add_traffic(&other.traffic_info);
        // Update other fields
        if self.hostname.is_empty() {
            self.hostname = other.hostname.clone();
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HostDisplayInfo {
    pub ip_addr: IpAddr,
    pub hostname: String,
    pub country_code: String,
    pub as_name: String,
    pub traffic: TrafficDisplayInfo,
}
