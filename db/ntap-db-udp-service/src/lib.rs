
use std::{collections::HashMap, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

#[cfg(feature = "bundle")]
pub const UDP_SERVICE_BIN: &[u8] = include_bytes!("../resources/udp-service.bin");

pub const UDP_SERVICE_BIN_NAME: &str = "udp-service.bin";
pub const UDP_SERVICE_R2_URL: &str = "https://r2.ntap.io/udp-service.bin";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UdpService {
    pub port: u16,
    pub service_name: String,
}

impl UdpService {
    pub fn file_name() -> String {
        UDP_SERVICE_BIN_NAME.to_owned()
    }
    pub fn r2_url() -> String {
        UDP_SERVICE_R2_URL.to_owned()
    }
}

#[cfg(feature = "bundle")]
pub fn get_map() -> HashMap<u16, String> {
    let mut udp_map: HashMap<u16, String> = HashMap::new();
    let udp_services: Vec<UdpService> = bincode::deserialize(UDP_SERVICE_BIN).unwrap();
    for udp_service in udp_services {
        udp_map.insert(udp_service.port, udp_service.service_name);
    }
    udp_map
}

pub fn get_map_from_file(file_path: PathBuf) -> HashMap<u16, String> {
    let mut udp_map: HashMap<u16, String> = HashMap::new();
    match fs::read(file_path) {
        Ok(f) => {
            let udp_services: Vec<UdpService> = bincode::deserialize(&f).unwrap();
            for udp_service in udp_services {
                udp_map.insert(udp_service.port, udp_service.service_name);
            }
        }
        Err(e) => {
            eprintln!("Error reading file: {}", e);
        }
    }
    udp_map
}
