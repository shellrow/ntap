use clap::{crate_name, crate_version};

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
