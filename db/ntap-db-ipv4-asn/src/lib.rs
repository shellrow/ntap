use std::{fs, path::PathBuf};
use rangemap::RangeInclusiveMap;
use serde::{Deserialize, Serialize};

#[cfg(feature = "bundle")]
pub const IPV4_ASN_BIN: &[u8] = include_bytes!("../resources/ipv4-asn.bin");

pub const IPV4_ASN_BIN_NAME: &str = "ipv4-asn.bin";
pub const IPV4_ASN_R2_URL: &str = "https://r2.ntap.io/ipv4-asn.bin";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Ipv4Asn {
    pub ip_from: u32,
    pub ip_to: u32,
    pub asn: u32,
}

impl Ipv4Asn {
    pub fn file_name() -> String {
        IPV4_ASN_BIN_NAME.to_owned()
    }
    pub fn r2_url() -> String {
        IPV4_ASN_R2_URL.to_owned()
    }
}

#[cfg(feature = "bundle")]
pub fn get_map() -> RangeInclusiveMap<u32, u32> {
    let mut ipv4_asn_map: RangeInclusiveMap<u32, u32> = RangeInclusiveMap::new();
    let ipv4_asn_vec: Vec<Ipv4Asn> = bincode::deserialize(IPV4_ASN_BIN).unwrap();
    for ipv4_asn in ipv4_asn_vec {
        ipv4_asn_map.insert(ipv4_asn.ip_from..=ipv4_asn.ip_to, ipv4_asn.asn);
    }
    ipv4_asn_map
}

pub fn get_map_from_file(file_path: PathBuf) -> RangeInclusiveMap<u32, u32> {
    let mut ipv4_asn_map: RangeInclusiveMap<u32, u32> = RangeInclusiveMap::new();
    match fs::read(file_path) {
        Ok(f) => {
            let ipv4_asn_vec: Vec<Ipv4Asn> = bincode::deserialize(&f).unwrap();
            for ipv4_asn in ipv4_asn_vec {
                ipv4_asn_map.insert(ipv4_asn.ip_from..=ipv4_asn.ip_to, ipv4_asn.asn);
            }
        }
        Err(e) => {
            eprintln!("Error reading file: {}", e);
        }
    }
    ipv4_asn_map
}
