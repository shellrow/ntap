use super::traffic::TrafficDisplayInfo;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServiceDisplayInfo {
    pub port: u16,
    pub protocol: String,
    pub name: String,
    pub traffic: TrafficDisplayInfo,
}
