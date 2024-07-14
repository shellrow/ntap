
use std::{collections::HashMap, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

#[cfg(feature = "bundle")]
pub const TCP_SERVICE_BIN: &[u8] = include_bytes!("../resources/tcp-service.bin");

pub const TCP_SERVICE_BIN_NAME: &str = "tcp-service.bin";
pub const TCP_SERVICE_R2_URL: &str = "https://r2.ntap.io/tcp-service.bin";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TcpService {
    pub port: u16,
    pub service_name: String,
}

impl TcpService {
    pub fn file_name() -> String {
        TCP_SERVICE_BIN_NAME.to_owned()
    }
    pub fn r2_url() -> String {
        TCP_SERVICE_R2_URL.to_owned()
    }
}

#[cfg(feature = "bundle")]
pub fn get_map() -> HashMap<u16, String> {
    let mut tcp_map: HashMap<u16, String> = HashMap::new();
    let tcp_services: Vec<TcpService> = bincode::deserialize(TCP_SERVICE_BIN).unwrap();
    for tcp_service in tcp_services {
        tcp_map.insert(tcp_service.port, tcp_service.service_name);
    }
    tcp_map
}

pub fn get_map_from_file(file_path: PathBuf) -> HashMap<u16, String> {
    let mut tcp_map: HashMap<u16, String> = HashMap::new();
    match fs::read(file_path) {
        Ok(f) => {
            let tcp_services: Vec<TcpService> = bincode::deserialize(&f).unwrap();
            for tcp_service in tcp_services {
                tcp_map.insert(tcp_service.port, tcp_service.service_name);
            }
        }
        Err(e) => {
            eprintln!("Error reading file: {}", e);
        }
    }
    tcp_map
}
