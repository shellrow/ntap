use crate::db;
use rangemap::RangeInclusiveMap;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};

/// In-memory IP database with range map and hash map
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IpDatabase {
    pub ipv4_country_map: RangeInclusiveMap<u32, String>,
    pub ipv6_country_map: RangeInclusiveMap<u128, String>,
    pub ipv4_asn_map: RangeInclusiveMap<u32, u32>,
    pub ipv6_asn_map: RangeInclusiveMap<u128, u32>,
    pub country_map: HashMap<String, String>,
    pub autonomous_map: HashMap<u32, String>,
}

impl IpDatabase {
    pub fn new() -> IpDatabase {
        IpDatabase {
            ipv4_country_map: RangeInclusiveMap::new(),
            ipv6_country_map: RangeInclusiveMap::new(),
            ipv4_asn_map: RangeInclusiveMap::new(),
            ipv6_asn_map: RangeInclusiveMap::new(),
            country_map: HashMap::new(),
            autonomous_map: HashMap::new(),
        }
    }
    pub fn load() -> Result<IpDatabase, Box<dyn std::error::Error>> {
        let mut ip_db = IpDatabase::new();
        ip_db.load_ipv4_country_map()?;
        ip_db.load_ipv6_country_map()?;
        ip_db.load_ipv4_asn_map()?;
        ip_db.load_ipv6_asn_map()?;
        ip_db.load_country_map()?;
        ip_db.load_autonomous_map()?;
        Ok(ip_db)
    }
    pub fn load_ipv4_country_map(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.ipv4_country_map = db::ipv4_country::get_map();
        Ok(())
    }
    pub fn load_ipv6_country_map(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.ipv6_country_map = db::ipv6_country::get_map();
        Ok(())
    }
    pub fn load_ipv4_asn_map(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.ipv4_asn_map = db::ipv4_asn::get_map();
        Ok(())
    }
    pub fn load_ipv6_asn_map(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.ipv6_asn_map = db::ipv6_asn::get_map();
        Ok(())
    }
    pub fn load_country_map(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.country_map = db::country::get_map();
        Ok(())
    }
    pub fn load_autonomous_map(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.autonomous_map = db::asn::get_map();
        Ok(())
    }
    pub fn get_ipv4_info(&self, ipv4_addr: Ipv4Addr) -> Option<IpInfo> {
        let mut ip_info = IpInfo::new(IpAddr::V4(ipv4_addr));
        let ip_addr_int: u64 = crate::net::ip::ipv4_to_int(ipv4_addr);
        ip_info.ip_version = String::from("v4");
        ip_info.ip_addr_dec = ip_addr_int.to_string();
        match self.ipv4_country_map.get(&(ip_addr_int as u32)) {
            Some(country_code) => {
                ip_info.country_code = country_code.to_string();
                match self.country_map.get(country_code) {
                    Some(country_name) => {
                        ip_info.country_name = country_name.to_string();
                    }
                    None => {}
                }
            }
            None => {}
        }
        match self.ipv4_asn_map.get(&(ip_addr_int as u32)) {
            Some(asn) => {
                ip_info.asn = *asn;
                match self.autonomous_map.get(asn) {
                    Some(as_name) => {
                        ip_info.as_name = as_name.to_string();
                    }
                    None => {}
                }
            }
            None => {}
        }
        if (ip_info.country_code.is_empty()
            || ip_info.country_code == "ZZ"
            || ip_info.country_code == "-")
            && ip_info.asn == 0
        {
            return None;
        } else {
            Some(ip_info)
        }
    }
    pub fn get_ipv6_info(&self, ip_addr: Ipv6Addr) -> Option<IpInfo> {
        let mut ip_info = IpInfo::new(IpAddr::V6(ip_addr));
        let ip_addr_int: u128 = crate::net::ip::ipv6_to_dec(ip_addr);
        ip_info.ip_version = String::from("v6");
        ip_info.ip_addr_dec = ip_addr_int.to_string();
        match self.ipv6_country_map.get(&(ip_addr_int)) {
            Some(country_code) => {
                ip_info.country_code = country_code.to_string();
                match self.country_map.get(country_code) {
                    Some(country_name) => {
                        ip_info.country_name = country_name.to_string();
                    }
                    None => {}
                }
            }
            None => {}
        }
        match self.ipv6_asn_map.get(&(ip_addr_int)) {
            Some(asn) => {
                ip_info.asn = *asn;
                match self.autonomous_map.get(asn) {
                    Some(as_name) => {
                        ip_info.as_name = as_name.to_string();
                    }
                    None => {}
                }
            }
            None => {}
        }
        if (ip_info.country_code.is_empty()
            || ip_info.country_code == "ZZ"
            || ip_info.country_code == "-")
            && ip_info.asn == 0
        {
            return None;
        } else {
            Some(ip_info)
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct IpInfo {
    pub ip_version: String,
    pub ip_addr_dec: String,
    pub ip_addr: IpAddr,
    pub host_name: String,
    pub asn: u32,
    pub as_name: String,
    pub country_code: String,
    pub country_name: String,
}

impl IpInfo {
    pub fn new(ip_addr: IpAddr) -> IpInfo {
        IpInfo {
            ip_version: String::new(),
            ip_addr_dec: String::new(),
            ip_addr: ip_addr,
            host_name: String::new(),
            asn: 0,
            as_name: String::new(),
            country_code: String::new(),
            country_name: String::new(),
        }
    }
}
