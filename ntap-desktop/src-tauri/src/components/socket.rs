use ntap_core::net::traffic::TrafficInfo;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServiceDisplayInfo {
    pub port: u16,
    pub name: String,
    pub traffic: TrafficInfo,
}
