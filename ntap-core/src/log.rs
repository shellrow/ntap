use serde::{Deserialize, Serialize};
use std::sync::{Mutex, OnceLock};

pub const DEFAULT_LOG_FILE_PATH: &str = "ntap.log";

/// Global Mutex lock guard for logging.
pub static LOG_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

/// Thread-safe logging macro. This macro is used to log messages from multiple threads.
#[macro_export]
macro_rules! thread_log {
    ($log_macro: ident, $($fmt_args:expr),*) => {{
        let guard = $crate::log::LOG_LOCK.get_or_init(|| std::sync::Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        log::$log_macro!($($fmt_args,)*);
        drop(guard);
    }};
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
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
    pub fn to_string(&self) -> String {
        match self {
            LogLevel::DEBUG => "DEBUG",
            LogLevel::INFO => "INFO",
            LogLevel::WARN => "WARN",
            LogLevel::ERROR => "ERROR",
        }
        .to_owned()
    }
    pub fn to_level_filter(&self) -> simplelog::LevelFilter {
        match self {
            LogLevel::DEBUG => simplelog::LevelFilter::Debug,
            LogLevel::INFO => simplelog::LevelFilter::Info,
            LogLevel::WARN => simplelog::LevelFilter::Warn,
            LogLevel::ERROR => simplelog::LevelFilter::Error,
        }
    }
}
