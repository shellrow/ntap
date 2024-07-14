use std::{collections::HashMap, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

#[cfg(feature = "bundle")]
pub const COUNTRY_BIN: &[u8] = include_bytes!("../resources/country.bin");

pub const COUNTRY_BIN_NAME: &str = "country.bin";
pub const COUNTRY_R2_URL: &str = "https://r2.ntap.io/country.bin";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Country {
    pub country_code: String,
    pub country_name: String,
}

impl Country {
    pub fn file_name() -> String {
        COUNTRY_BIN_NAME.to_owned()
    }
    pub fn r2_url() -> String {
        COUNTRY_R2_URL.to_owned()
    }
}

#[cfg(feature = "bundle")]
pub fn get_map() -> HashMap<String, String> {
    let mut country_map: HashMap<String, String> = HashMap::new();
    let country_vec: Vec<Country> = bincode::deserialize(COUNTRY_BIN).unwrap();
    for country in country_vec {
        country_map.insert(country.country_code, country.country_name);
    }
    country_map
}

pub fn get_map_from_file(file_path: PathBuf) -> HashMap<String, String> {
    let mut country_map: HashMap<String, String> = HashMap::new();
    match fs::read(file_path) {
        Ok(f) => {
            let country_vec: Vec<Country> = bincode::deserialize(&f).unwrap();
            for country in country_vec {
                country_map.insert(country.country_code, country.country_name);
            }
        }
        Err(e) => {
            eprintln!("Error reading file: {}", e);
        }
    }
    country_map
}
