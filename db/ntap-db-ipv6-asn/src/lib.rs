use std::{fs, path::PathBuf};
use rangemap::RangeInclusiveMap;
use serde::{Deserialize, Serialize};

#[cfg(feature = "bundle")]
pub const IPV6_ASN_BIN: &[u8] = include_bytes!("../resources/ipv6-asn.bin");

pub const IPV6_ASN_BIN_NAME: &str = "ipv6-asn.bin";
pub const IPV6_ASN_R2_URL: &str = "https://r2.ntap.io/ipv6-asn.bin";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Ipv6Asn {
    pub ip_from: u128,
    pub ip_to: u128,
    pub asn: u32,
}

impl Ipv6Asn {
    pub fn file_name() -> String {
        IPV6_ASN_BIN_NAME.to_owned()
    }
    pub fn r2_url() -> String {
        IPV6_ASN_R2_URL.to_owned()
    }
}

#[cfg(feature = "bundle")]
pub fn get_map() -> RangeInclusiveMap<u128, u32> {
    let mut ipv6_asn_map: RangeInclusiveMap<u128, u32> = RangeInclusiveMap::new();
    let ipv6_asn_vec: Vec<Ipv6Asn> = bincode::deserialize(IPV6_ASN_BIN).unwrap();
    for ipv6_asn in ipv6_asn_vec {
        ipv6_asn_map.insert(ipv6_asn.ip_from..=ipv6_asn.ip_to, ipv6_asn.asn);
    }
    ipv6_asn_map
}

pub fn get_map_from_file(file_path: PathBuf) -> RangeInclusiveMap<u128, u32> {
    let mut ipv6_asn_map: RangeInclusiveMap<u128, u32> = RangeInclusiveMap::new();
    match fs::read(file_path) {
        Ok(f) => {
            let ipv6_asn_vec: Vec<Ipv6Asn> = bincode::deserialize(&f).unwrap();
            for ipv6_asn in ipv6_asn_vec {
                ipv6_asn_map.insert(ipv6_asn.ip_from..=ipv6_asn.ip_to, ipv6_asn.asn);
            }
        }
        Err(e) => {
            eprintln!("Error reading file: {}", e);
        }
    }
    ipv6_asn_map
}
