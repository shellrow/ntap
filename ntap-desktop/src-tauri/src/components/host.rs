use ntap_core::net::traffic::TrafficInfo;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HostDisplayInfo {
    pub ip_addr: IpAddr,
    pub host_name: String,
    pub country_code: String,
    pub country_name: String,
    pub asn: u32,
    pub as_name: String,
    pub traffic: TrafficInfo,
}
