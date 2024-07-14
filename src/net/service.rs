use serde::{Deserialize, Serialize};
use super::traffic::TrafficDisplayInfo;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServiceDisplayInfo {
    pub port: u16,
    pub protocol: String,
    pub name: String,
    pub traffic: TrafficDisplayInfo,
}
