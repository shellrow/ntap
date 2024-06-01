use super::host::HostDisplayInfo;
use super::process::ProcessDisplayInfo;
use super::socket::ServiceDisplayInfo;
use ntap_core::net::traffic::TrafficInfo;
use ntap_core::notification::Notification;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Overview {
    pub default_if_index: u32,
    pub default_if_name: String,
    pub captured_packets: usize,
    pub traffic: TrafficInfo,
    pub top_processes: Vec<ProcessDisplayInfo>,
    pub top_remote_hosts: Vec<HostDisplayInfo>,
    pub top_app_protocols: Vec<ServiceDisplayInfo>,
    pub notificatons: Vec<Notification>,
}
