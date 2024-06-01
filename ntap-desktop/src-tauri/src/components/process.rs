use ntap_core::net::traffic::TrafficInfo;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessDisplayInfo {
    pub pid: u32,
    pub name: String,
    pub traffic: TrafficInfo,
}
