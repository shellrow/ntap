use std::fs::File;
use std::path::Path;
use std::error::Error;
use ntap_core::config::AppConfig;

#[cfg(not(target_os = "windows"))]
mod unix;
#[allow(unused_imports)]
#[cfg(not(target_os = "windows"))]
pub use self::unix::*;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use self::windows::*;

pub fn init(_handle: &tauri::AppHandle) -> Result<(), Box<dyn Error>> {
    log::info!("Init ntap-desktop");
    // Check .ntap directory
    match ntap_core::sys::get_config_dir_path() {
        Some(_config_dir) => {
            // TODO!
        }
        None => {
            return Err("Error: Could not get config directory path".into());
        }
    }

    // Load AppConfig
    let config = AppConfig::load();

    // Init logger
    let log_file_path = if let Some(file_path) = &config.logging.file_path {
        // Convert to PathBuf
        Path::new(&file_path).to_path_buf()
    } else {
        ntap_core::sys::get_user_file_path(ntap_core::log::DEFAULT_LOG_FILE_PATH).unwrap()
    };
    let log_file: File = if log_file_path.exists() {
        File::options().write(true).open(&log_file_path)?
    } else {
        File::create(&log_file_path)?
    };
    let mut log_config_builder = simplelog::ConfigBuilder::default();
    log_config_builder.set_time_format_rfc3339();
    if let Some(offset) = ntap_core::time::get_local_offset() {
        log_config_builder.set_time_offset(offset);
    }
    let default_log_config = log_config_builder.build();
    simplelog::CombinedLogger::init(vec![
        simplelog::TermLogger::new(
            simplelog::LevelFilter::Info,
            default_log_config.clone(),
            simplelog::TerminalMode::Mixed,
            simplelog::ColorChoice::Auto,
        ),
        simplelog::WriteLogger::new(
            config.logging.level.to_level_filter(),
            default_log_config,
            log_file,
        ),
    ])?;
    log::info!("Init complete");
    Ok(())
}

pub fn cleanup() {
    log::info!("Cleanup");
}
