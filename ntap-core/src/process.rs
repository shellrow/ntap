use serde::{Deserialize, Serialize};

use crate::net::traffic::{TrafficDisplayInfo, TrafficInfo};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserInfo {
    pub user_id: String,
    pub group_id: String,
    pub user_name: String,
    pub groups: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
}

impl ProcessInfo {
    pub fn new(pid: u32, name: String) -> ProcessInfo {
        ProcessInfo {
            pid: pid,
            name: name,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessTrafficInfo {
    pub process: ProcessInfo,
    pub traffic: TrafficInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessDisplayInfo {
    pub pid: u32,
    pub name: String,
    pub traffic: TrafficDisplayInfo,
}
