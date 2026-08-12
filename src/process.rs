use serde::{Deserialize, Serialize};

use crate::net::traffic::TrafficDisplayInfo;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
}

impl ProcessInfo {
    pub fn new(pid: u32, name: String) -> ProcessInfo {
        ProcessInfo { pid, name }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessDisplayInfo {
    pub pid: u32,
    pub name: String,
    pub traffic: TrafficDisplayInfo,
}
