use super::DepsError;
use crate::sys;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::path::PathBuf;

pub const NPCAP_SOFTWARE_NAME: &str = "Npcap";
pub const NPCAP_INSTALL_DIR_NAME: &str = "npcap";
pub const NPCAP_SDK_DIR_NAME: &str = "npcap-sdk-1.13";
pub const NPCAP_INSTALLER_FILENAME: &str = "npcap-1.79.exe";
pub const NPCAP_SDK_FILENAME: &str = "npcap-sdk-1.13.zip";
pub const NPCAP_INSTALLER_HASH: &str =
    "A95577EBBC67FC45B319E2EF3A55F4E9B211FE82ED4CB9D8BE6B1A9E2425CE53";
pub const NPCAP_SDK_HASH: &str = "DAD1F2BF1B02B787BE08CA4862F99E39A876C1F274BAC4AC0CEDC9BBC58F94FD";
pub const NPCAP_DIST_BASE_URL: &str = "https://npcap.com/dist/";
pub const NPCAP_LIB_NAME: &str = "Packet.lib";

pub fn check_deps() -> Result<(), DepsError> {
    check_npcap()
}

// Check dependencies and return a map of dependencies and their status
pub fn get_deps_map() -> HashMap<String, bool> {
    let mut deps_map: HashMap<String, bool> = HashMap::new();
    deps_map.insert(NPCAP_SOFTWARE_NAME.to_lowercase(), npcap_installed());
    deps_map
}

pub fn check_npcap() -> Result<(), DepsError> {
    if npcap_installed() {
        Ok(())
    } else {
        Err(DepsError::Missing(NPCAP_SOFTWARE_NAME.to_owned()))
    }
}

/// Check if npcap is installed.
/// This function only check if npcap is installed, not check version.
pub fn npcap_installed() -> bool {
    sys::software_installed(NPCAP_SOFTWARE_NAME.to_owned())
}

/// Check if npcap SDK is installed.
/// This function only check if npcap SDK is installed, not check version.
pub fn npcap_sdk_installed() -> bool {
    let env_lib_value: String = sys::get_env_lib();
    if env_lib_value.is_empty() {
        return false;
    }
    // Split env_lib_value by ;
    let lib_path_list: Vec<&str> = env_lib_value.split(";").collect();
    // Check if npcap sdk is in env_lib_value
    // Search for Packet.lib
    for lib_path in lib_path_list {
        let packet_lib_path: String = format!("{}\\{}", lib_path, NPCAP_LIB_NAME);
        if std::path::Path::new(&packet_lib_path).exists() {
            return true;
        }
    }
    false
}
