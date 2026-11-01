use winreg::RegKey;
use winreg::enums::HKEY_LOCAL_MACHINE;

pub fn software_installed(software_name: &str) -> bool {
    let hklm: RegKey = RegKey::predef(HKEY_LOCAL_MACHINE);
    [
        format!("SOFTWARE\\{software_name}"),
        format!("SOFTWARE\\WOW6432Node\\{software_name}"),
    ]
    .iter()
    .any(|path| hklm.open_subkey(path).is_ok())
}

pub fn check_deps() -> anyhow::Result<()> {
    if software_installed("Npcap") {
        Ok(())
    } else {
        anyhow::bail!(
            "Npcap is required on Windows; install it in WinPcap API-compatible mode and retry"
        )
    }
}
