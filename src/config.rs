use crate::sys;
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::io::ErrorKind;

pub const NTAP_CONFIG_FILE_NAME: &str = "ntap-config.json";
pub const DEFAULT_LOG_FILE_PATH: &str = "ntap.log";

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(default)]
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
        Self::default()
    }
    pub fn load() -> Result<AppConfig> {
        let path = sys::get_user_file_path(NTAP_CONFIG_FILE_NAME)
            .context("could not resolve the ntap configuration path")?;
        match std::fs::read_to_string(&path) {
            Ok(content) => {
                let config: AppConfig = serde_json::from_str(&content)
                    .with_context(|| format!("invalid configuration file: {}", path.display()))?;
                config.validate()?;
                Ok(config)
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {
                let config = AppConfig::new();
                config.save()?;
                Ok(config)
            }
            Err(error) => Err(error)
                .with_context(|| format!("failed to read configuration: {}", path.display())),
        }
    }
    pub fn save(&self) -> Result<()> {
        self.validate()?;
        let path = sys::get_user_file_path(NTAP_CONFIG_FILE_NAME)
            .context("could not resolve the ntap configuration path")?;
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)
            .with_context(|| format!("failed to write configuration: {}", path.display()))
    }

    pub fn validate(&self) -> Result<()> {
        if !(16..=60_000).contains(&self.display.tick_rate) {
            bail!("display.tick_rate must be between 16 and 60000 milliseconds");
        }
        if self.network.entry_ttl < 100 {
            bail!("network.entry_ttl must be at least 100 milliseconds");
        }
        if self.display.top_remote_hosts == 0 || self.display.connection_count == 0 {
            bail!("display row limits must be greater than zero");
        }
        Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Default)]
#[allow(clippy::upper_case_acronyms)]
pub enum LogLevel {
    DEBUG,
    #[default]
    INFO,
    WARN,
    ERROR,
}

impl LogLevel {
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
#[serde(default)]
pub struct LoggingConfig {
    /// Log level.
    pub level: LogLevel,
    /// Log file path.
    pub file_path: Option<String>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self::new()
    }
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
#[serde(default)]
pub struct NetworkConfig {
    /// Network interfaces to use. If empty, all interfaces will be use.
    pub interfaces: Vec<String>,
    /// Enable reverse DNS lookup.
    pub reverse_dns: bool,
    /// Entry TTL in milliseconds. If no traffic is seen for this duration, the entry will be removed.
    pub entry_ttl: u64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self::new()
    }
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
#[serde(default)]
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

impl Default for DisplayConfig {
    fn default() -> Self {
        Self::new()
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_json_fields_receive_defaults() {
        let config: AppConfig = serde_json::from_str(r#"{"network":{"reverse_dns":true}}"#)
            .expect("partial configuration should deserialize");
        assert!(config.network.reverse_dns);
        assert_eq!(config.display.tick_rate, 1_000);
        assert_eq!(config.network.entry_ttl, 60_000);
    }

    #[test]
    fn unsafe_intervals_are_rejected() {
        let mut config = AppConfig::default();
        config.display.tick_rate = 0;
        assert!(config.validate().is_err());
        config.display.tick_rate = 1_000;
        config.network.entry_ttl = 0;
        assert!(config.validate().is_err());
    }
}
