use std::{fs, path::PathBuf};
use rangemap::RangeInclusiveMap;
use serde::{Deserialize, Serialize};

#[cfg(feature = "bundle")]
pub const IPV6_COUNTRY_BIN: &[u8] = include_bytes!("../resources/ipv6-country.bin");

pub const IPV6_COUNTRY_BIN_NAME: &str = "ipv6-country.bin";
pub const IPV6_COUNTRY_R2_URL: &str = "https://r2.ntap.io/ipv6-country.bin";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Ipv6Country {
    pub ip_from: u128,
    pub ip_to: u128,
    pub country_code: String,
}

impl Ipv6Country {
    pub fn file_name() -> String {
        IPV6_COUNTRY_BIN_NAME.to_owned()
    }
    pub fn r2_url() -> String {
        IPV6_COUNTRY_R2_URL.to_owned()
    }
}

#[cfg(feature = "bundle")]
pub fn get_map() -> RangeInclusiveMap<u128, String> {
    let mut ipv6_country_map: RangeInclusiveMap<u128, String> = RangeInclusiveMap::new();
    let ipv6_country_vec: Vec<Ipv6Country> = bincode::deserialize(IPV6_COUNTRY_BIN).unwrap();
    for ipv6_country in ipv6_country_vec {
        ipv6_country_map.insert(ipv6_country.ip_from..=ipv6_country.ip_to, ipv6_country.country_code);
    }
    ipv6_country_map
}

pub fn get_map_from_file(file_path: PathBuf) -> RangeInclusiveMap<u128, String> {
    let mut ipv6_country_map: RangeInclusiveMap<u128, String> = RangeInclusiveMap::new();
    match fs::read(file_path) {
        Ok(f) => {
            let ipv6_country_vec: Vec<Ipv6Country> = bincode::deserialize(&f).unwrap();
            for ipv6_country in ipv6_country_vec {
                ipv6_country_map.insert(ipv6_country.ip_from..=ipv6_country.ip_to, ipv6_country.country_code);
            }
        }
        Err(e) => {
            eprintln!("Error reading file: {}", e);
        }
    }
    ipv6_country_map
}
