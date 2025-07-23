use clap::{crate_name, crate_version};
use std::path::PathBuf;

#[cfg(not(target_os = "windows"))]
mod unix;
#[cfg(not(target_os = "windows"))]
pub use self::unix::*;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use self::windows::*;

pub fn get_app_title() -> String {
    format!("{} v{}", crate_name!(), crate_version!())
}

pub const USER_CONFIG_DIR_NAME: &str = ".ntap";

pub fn get_sysdate() -> String {
    let now = chrono::Local::now();
    now.to_rfc3339()
}

pub fn get_config_dir_path() -> Option<PathBuf> {
    match home::home_dir() {
        Some(mut path) => {
            path.push(USER_CONFIG_DIR_NAME);
            if !path.exists() {
                match std::fs::create_dir_all(&path) {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!("Failed to create config dir: {:?}", e);
                        return None;
                    }
                }
            }
            Some(path)
        }
        None => None,
    }
}

pub fn get_user_file_path(file_name: &str) -> Option<PathBuf> {
    match get_config_dir_path() {
        Some(mut path) => {
            path.push(file_name);
            Some(path)
        }
        None => None,
    }
}


pub fn get_database_dir_path() -> Option<PathBuf> {
    match get_config_dir_path() {
        Some(mut path) => {
            path.push("db");
            if !path.exists() {
                match std::fs::create_dir_all(&path) {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!("Failed to create database dir: {:?}", e);
                        return None;
                    }
                }
            }
            Some(path)
        }
        None => None,
    }
}
