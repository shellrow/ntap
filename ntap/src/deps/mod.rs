#![allow(unused)]

#[cfg(not(target_os = "windows"))]
mod unix;
#[cfg(not(target_os = "windows"))]
pub use self::unix::*;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use self::windows::*;

use serde::{Deserialize, Serialize};
use std::fmt;

// Custom error type for check dependencies
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum DepsError {
    Missing(String),
    Unknown(String),
}

impl fmt::Display for DepsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DepsError::Missing(ref s) => write!(f, "missing: {}", s),
            DepsError::Unknown(ref s) => write!(f, "unknown: {}", s),
        }
    }
}

// Check Database files
pub fn check_db_files() -> Result<(), DepsError> {
    let db_files = vec![
        ntap_db_as::AS_BIN_NAME,
        ntap_db_country::COUNTRY_BIN_NAME,
        ntap_db_ipv4_asn::IPV4_ASN_BIN_NAME,
        ntap_db_ipv4_country::IPV4_COUNTRY_BIN_NAME,
        ntap_db_ipv6_asn::IPV6_ASN_BIN_NAME,
        ntap_db_ipv6_country::IPV6_COUNTRY_BIN_NAME,
        ntap_db_oui::OUI_BIN_NAME,
        ntap_db_tcp_service::TCP_SERVICE_BIN_NAME,
        ntap_db_udp_service::UDP_SERVICE_BIN_NAME,
    ];
    for file in db_files {
        match crate::sys::get_db_file_path(file) {
            Some(file_path) => {
                if !file_path.exists() {
                    return Err(DepsError::Missing(file.to_string()));
                }
            }
            None => {
                return Err(DepsError::Unknown(file.to_string()));
            }
        }
    }
    Ok(())
}
