use super::traffic::{TrafficDisplayInfo, TrafficInfo};
use serde::{Deserialize, Serialize};
use std::{net::IpAddr, time::Duration};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RemoteHostInfo {
    pub mac_addr: String,
    pub ip_addr: IpAddr,
    pub hostname: String,
    pub country_code: String,
    pub country_name: String,
    pub asn: u32,
    pub as_name: String,
    pub traffic_info: TrafficInfo,
}

impl RemoteHostInfo {
    pub fn new(mac_addr: String, ip_addr: IpAddr) -> Self {
        RemoteHostInfo {
            mac_addr: mac_addr,
            ip_addr: ip_addr,
            hostname: String::new(),
            country_code: String::new(),
            country_name: String::new(),
            asn: 0,
            as_name: String::new(),
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
        if self.country_code.is_empty() {
            self.country_code = other.country_code.clone();
        }
        if self.country_name.is_empty() {
            self.country_name = other.country_name.clone();
        }
        if self.asn == 0 {
            self.asn = other.asn;
        }
        if self.as_name.is_empty() {
            self.as_name = other.as_name.clone();
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HostDisplayInfo {
    pub ip_addr: IpAddr,
    pub host_name: String,
    pub country_code: String,
    pub country_name: String,
    pub asn: u32,
    pub as_name: String,
    pub traffic: TrafficDisplayInfo,
}