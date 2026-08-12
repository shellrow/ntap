use anyhow::Result;
use std::fs::{File, OpenOptions};
use std::path::Path;
use tracing_subscriber::FmtSubscriber;
use tracing_subscriber::fmt::time::ChronoLocal;

pub fn init_logger(config: &crate::config::AppConfig) -> Result<()> {
    // Init logger
    let log_file_path = if let Some(file_path) = &config.logging.file_path {
        // Convert to PathBuf
        Path::new(&file_path).to_path_buf()
    } else {
        crate::sys::get_user_file_path(crate::config::DEFAULT_LOG_FILE_PATH)
            .ok_or_else(|| anyhow::anyhow!("failed to resolve default log file path"))?
    };
    let log_file: File = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_path)?;
    let subscriber = FmtSubscriber::builder()
        .with_max_level(config.logging.level.to_level_filter())
        .with_ansi(false)
        .with_target(false)
        .with_timer(ChronoLocal::rfc_3339())
        .with_writer(std::sync::Arc::new(log_file))
        .finish();
    if let Err(error) = tracing::subscriber::set_global_default(subscriber) {
        eprintln!("logger already initialized or unavailable: {error}");
    }

    Ok(())
}
