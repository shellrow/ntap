use serde::{Deserialize, Serialize};

// env
pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
pub const APP_DESCRIPTION: &str = "Real-time network utilization monitoring tool";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_RELEASE_DATE: &str = "2024-06-11";
pub const APP_REPOSITORY : &str = env!("CARGO_PKG_REPOSITORY");

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppInfo {
    pub name: String,
    pub description: String,
    pub version: String,
    pub release_date: String,
    pub repository: String,
}

impl AppInfo {
    pub fn new() -> AppInfo {
        AppInfo {
            name: APP_NAME.to_string(),
            description: APP_DESCRIPTION.to_string(),
            version: APP_VERSION.to_string(),
            release_date: APP_RELEASE_DATE.to_string(),
            repository: APP_REPOSITORY.to_string(),
        }
    }
}
