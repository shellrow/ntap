use super::DepsError;
use crate::sys;

pub const NPCAP_SOFTWARE_NAME: &str = "Npcap";

pub fn check_deps() -> Result<(), DepsError> {
    check_npcap()
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
