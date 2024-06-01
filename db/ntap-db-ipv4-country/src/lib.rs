use std::{fs, path::PathBuf};
use rangemap::RangeInclusiveMap;
use serde::{Deserialize, Serialize};

#[cfg(feature = "bundle")]
pub const IPV4_COUNTRY_BIN: &[u8] = include_bytes!("../resources/ipv4-country.bin");

pub const IPV4_COUNTRY_BIN_NAME: &str = "ipv4-country.bin";
pub const IPV4_COUNTRY_R2_URL: &str = "https://r2.ntap.io/ipv4-country.bin";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Ipv4Country {
    pub ip_from: u32,
    pub ip_to: u32,
    pub country_code: String,
}

impl Ipv4Country {
    pub fn file_name() -> String {
        IPV4_COUNTRY_BIN_NAME.to_owned()
    }
    pub fn r2_url() -> String {
        IPV4_COUNTRY_R2_URL.to_owned()
    }
}

#[cfg(feature = "bundle")]
pub fn get_map() -> RangeInclusiveMap<u32, String> {
    let mut ipv4_country_map: RangeInclusiveMap<u32, String> = RangeInclusiveMap::new();
    let ipv4_country_vec: Vec<Ipv4Country> = bincode::deserialize(IPV4_COUNTRY_BIN).unwrap();
    for ipv4_country in ipv4_country_vec {
        ipv4_country_map.insert(ipv4_country.ip_from..=ipv4_country.ip_to, ipv4_country.country_code);
    }
    ipv4_country_map
}

pub fn get_map_from_file(file_path: PathBuf) -> RangeInclusiveMap<u32, String> {
    let mut ipv4_country_map: RangeInclusiveMap<u32, String> = RangeInclusiveMap::new();
    match fs::read(file_path) {
        Ok(f) => {
            let ipv4_country_vec: Vec<Ipv4Country> = bincode::deserialize(&f).unwrap();
            for ipv4_country in ipv4_country_vec {
                ipv4_country_map.insert(ipv4_country.ip_from..=ipv4_country.ip_to, ipv4_country.country_code);
            }
        }
        Err(e) => {
            eprintln!("Error reading file: {}", e);
        }
    }
    ipv4_country_map
}
