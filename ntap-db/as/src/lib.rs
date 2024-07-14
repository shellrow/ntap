use std::{collections::HashMap, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

#[cfg(feature = "bundle")]
pub const AS_BIN: &[u8] = include_bytes!("../resources/as.bin");

pub const AS_BIN_NAME: &str = "as.bin";
pub const AS_R2_URL: &str = "https://r2.ntap.io/as.bin";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AutonomousSystem {
    pub asn: u32,
    pub as_name: String,
}

impl AutonomousSystem {
    pub fn file_name() -> String {
        AS_BIN_NAME.to_owned()
    }
    pub fn r2_url() -> String {
        AS_R2_URL.to_owned()
    }
}

#[cfg(feature = "bundle")]
pub fn get_map() -> HashMap<u32, String> {
    let mut autonomous_map: HashMap<u32, String> = HashMap::new();
    let autonomous_vec: Vec<AutonomousSystem> = bincode::deserialize(AS_BIN).unwrap();
    for autonomous in autonomous_vec {
        autonomous_map.insert(autonomous.asn, autonomous.as_name);
    }
    autonomous_map
}

pub fn get_map_from_file(file_path: PathBuf) -> HashMap<u32, String> {
    let mut autonomous_map: HashMap<u32, String> = HashMap::new();
    match fs::read(file_path) {
        Ok(f) => {
            let autonomous_vec: Vec<AutonomousSystem> = bincode::deserialize(&f).unwrap();
            for autonomous in autonomous_vec {
                autonomous_map.insert(autonomous.asn, autonomous.as_name);
            }
        }
        Err(e) => {
            eprintln!("Error reading file: {}", e);
        }
    }
    autonomous_map
}
