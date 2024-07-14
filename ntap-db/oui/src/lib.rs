use std::{collections::HashMap, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

#[cfg(feature = "bundle")]
pub const OUI_BIN: &[u8] = include_bytes!("../resources/oui.bin");

pub const OUI_BIN_NAME: &str = "oui.bin";
pub const OUI_R2_URL: &str = "https://r2.ntap.io/oui.bin";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Oui {
    pub mac_prefix: String,
    pub vendor_name: String,
}

impl Oui {
    pub fn file_name() -> String {
        OUI_BIN_NAME.to_owned()
    }
    pub fn r2_url() -> String {
        OUI_R2_URL.to_owned()
    }
}

#[cfg(feature = "bundle")]
pub fn get_map() -> HashMap<String, String> {
    let mut oui_map: HashMap<String, String> = HashMap::new();
    let oui_vec: Vec<Oui> = bincode::deserialize(OUI_BIN).unwrap();
    for oui in oui_vec {
        oui_map.insert(oui.mac_prefix, oui.vendor_name);
    }
    oui_map
}

pub fn get_map_from_file(file_path: PathBuf) -> HashMap<String, String> {
    let mut oui_map: HashMap<String, String> = HashMap::new();
    match fs::read(file_path) {
        Ok(f) => {
            let oui_vec: Vec<Oui> = bincode::deserialize(&f).unwrap();
            for oui in oui_vec {
                oui_map.insert(oui.mac_prefix, oui.vendor_name);
            }
        }
        Err(e) => {
            eprintln!("Error reading file: {}", e);
        }
    }
    oui_map
}
