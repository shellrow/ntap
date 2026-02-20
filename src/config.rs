#![allow(unused)]

use crate::sys;
use serde::{Deserialize, Serialize};

pub const NTAP_CONFIG_FILE_NAME: &str = "ntap-config.json";
pub const DEFAULT_LOG_FILE_PATH: &str = "ntap.log";

#[derive(Deserialize, Serialize, Debug)]
pub struct AppConfig {
    /// Logging configuration.
    pub logging: LoggingConfig,
    /// Network configuration.
    pub network: NetworkConfig,
    /// Display configuration.
    pub display: DisplayConfig,
}

impl AppConfig {
    pub fn new() -> AppConfig {
        AppConfig {
            logging: LoggingConfig::new(),
            network: NetworkConfig::new(),
            display: DisplayConfig::new(),
        }
    }
    pub fn load() -> AppConfig {
        match sys::get_user_file_path(NTAP_CONFIG_FILE_NAME) {
            Some(path) => {
                match std::fs::read_to_string(&path) {
                    Ok(content) => match serde_json::from_str(&content) {
                        Ok(config) => config,
                        Err(e) => {
                            tracing::error!("{:?}", e);
                            AppConfig::new()
                        }
                    },
                    Err(e) => {
                        tracing::error!("{:?}", e);
                        // Create default config
                        let config = AppConfig::new();
                        config.save();
                        config
                    }
                }
            }
            None => {
                // Create default config
                let config = AppConfig::new();
                config.save();
                config
            }
        }
    }
    pub fn save(&self) {
        if let Some(path) = sys::get_user_file_path(NTAP_CONFIG_FILE_NAME) {
            match serde_json::to_string_pretty(&self) {
                Ok(content) => match std::fs::write(&path, content) {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!("{:?}", e);
                    }
                },
                Err(e) => {
                    tracing::error!("{:?}", e);
                }
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub enum LogLevel {
    DEBUG,
    INFO,
    WARN,
    ERROR,
}

impl LogLevel {
    pub fn allows(&self, level: &LogLevel) -> bool {
        match self {
            LogLevel::DEBUG => true,
            LogLevel::INFO => level != &LogLevel::DEBUG,
            LogLevel::WARN => level == &LogLevel::WARN || level == &LogLevel::ERROR,
            LogLevel::ERROR => level == &LogLevel::ERROR,
        }
    }
    pub fn to_level_filter(&self) -> tracing::Level {
        match self {
            LogLevel::DEBUG => tracing::Level::DEBUG,
            LogLevel::INFO => tracing::Level::INFO,
            LogLevel::WARN => tracing::Level::WARN,
            LogLevel::ERROR => tracing::Level::ERROR,
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            LogLevel::DEBUG => "DEBUG",
            LogLevel::INFO => "INFO",
            LogLevel::WARN => "WARN",
            LogLevel::ERROR => "ERROR",
        };
        write!(f, "{s}")
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LoggingConfig {
    /// Log level.
    pub level: LogLevel,
    /// Log file path.
    pub file_path: Option<String>,
}

impl LoggingConfig {
    pub fn new() -> LoggingConfig {
        LoggingConfig {
            level: LogLevel::INFO,
            file_path: sys::get_user_file_path(DEFAULT_LOG_FILE_PATH)
                .map(|path| path.to_string_lossy().to_string()),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NetworkConfig {
    /// Network interfaces to use. If empty, all interfaces will be use.
    pub interfaces: Vec<String>,
    /// Enable reverse DNS lookup.
    pub reverse_dns: bool,
    /// Entry TTL in milliseconds. If no traffic is seen for this duration, the entry will be removed.
    pub entry_ttl: u64,
}

impl NetworkConfig {
    pub fn new() -> NetworkConfig {
        NetworkConfig {
            interfaces: Vec::new(),
            reverse_dns: false,
            entry_ttl: 60000,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DisplayConfig {
    /// The number of top remote hosts to display in the Overview tab.
    pub top_remote_hosts: usize,
    /// The number of connections to display in the Overview tab.
    pub connection_count: usize,
    /// The tick rate in milliseconds. Default is 1000.
    /// This is the default rate at which the UI will update.
    pub tick_rate: u64,
    /// Show traffic as total or bandwidth.
    /// true: bandwidth, false: total.
    /// Default is total.
    pub show_bandwidth: bool,
}

impl DisplayConfig {
    pub fn new() -> DisplayConfig {
        DisplayConfig {
            top_remote_hosts: 20,
            connection_count: 20,
            tick_rate: 1000,
            show_bandwidth: false,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DatabaseConfig {
    pub oui_db_path: String,
    pub tcp_service_db_path: String,
    pub udp_service_db_path: String,
}

impl DatabaseConfig {
    pub fn new() -> DatabaseConfig {
        DatabaseConfig {
            oui_db_path: String::new(),
            tcp_service_db_path: String::new(),
            udp_service_db_path: String::new(),
        }
    }
}
