use inquire::Confirm;
use winreg::RegKey;
use winreg::enums::HKEY_LOCAL_MACHINE;

use crate::deps::DepsError;

pub fn get_os_bit() -> String {
    if cfg!(target_pointer_width = "32") {
        return "32-bit".to_owned();
    } else if cfg!(target_pointer_width = "64") {
        return "64-bit".to_owned();
    } else {
        return "unknown".to_owned();
    }
}

// Get software installation status
pub fn software_installed(software_name: String) -> bool {
    let hklm: RegKey = RegKey::predef(HKEY_LOCAL_MACHINE);
    let os_bit: String = get_os_bit();
    let npcap_key: RegKey = if os_bit == "32-bit" {
        match hklm.open_subkey(format!("SOFTWARE\\{}", software_name)) {
            Ok(key) => key,
            Err(_) => return false,
        }
    } else {
        match hklm.open_subkey(format!("SOFTWARE\\WOW6432Node\\{}", software_name)) {
            Ok(key) => key,
            Err(_) => return false,
        }
    };
    let _version: String = npcap_key.get_value("").unwrap_or(String::new());
    true
}

pub fn check_deps() -> Result<(), DepsError> {
    match crate::deps::check_deps() {
        Ok(_) => {
            return Ok(());
        }
        Err(e) => match e {
            crate::deps::DepsError::Missing(s) => {
                if s == crate::deps::NPCAP_SOFTWARE_NAME.to_string() {
                    let ans: bool = Confirm::new(
                        "Npcap is not installed, would you like to download & install it ?",
                    )
                    .prompt()
                    .unwrap();
                    if ans == false {
                        return Err(DepsError::Missing("On windows, Npcap is required for ntap to work properly. Please install Npcap and try again.".to_string()));
                    }
                }
            }
            crate::deps::DepsError::Unknown(s) => {
                eprintln!("Error: Unknown dependency: {}", s);
            }
        },
    }
    Ok(())
}
