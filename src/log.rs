use anyhow::Result;
use std::fs::File;
use std::path::Path;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use tracing_subscriber::fmt::time::ChronoLocal;
use tracing_subscriber::fmt::writer::BoxMakeWriter;
use tracing_subscriber::fmt::writer::MakeWriterExt;

pub fn init_logger(config: &crate::config::AppConfig) -> Result<()> {
    // Init logger
    let log_file_path = if let Some(file_path) = &config.logging.file_path {
        // Convert to PathBuf
        Path::new(&file_path).to_path_buf()
    } else {
        crate::sys::get_user_file_path(crate::config::DEFAULT_LOG_FILE_PATH)
            .ok_or_else(|| anyhow::anyhow!("failed to resolve default log file path"))?
    };
    let log_file: File = if log_file_path.exists() {
        File::options().write(true).open(&log_file_path)?
    } else {
        File::create(&log_file_path)?
    };
    let error_log = std::sync::Arc::new(log_file);

    if cfg!(debug_assertions) {
        let error_writer = error_log.with_max_level(Level::ERROR);
        let else_writer = BoxMakeWriter::new(std::io::stdout);
        let writer = error_writer.and(else_writer);
        let subscriber = FmtSubscriber::builder()
            .with_max_level(config.logging.level.to_level_filter())
            .with_ansi(false)
            .with_target(false)
            .with_timer(ChronoLocal::rfc_3339())
            .with_writer(writer)
            .finish();
        if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
            eprintln!("logger already initialized or unavailable: {}", e);
        }
    } else {
        // In release mode, log only to the error log file
        let error_writer = error_log.with_max_level(Level::ERROR);
        let subscriber = FmtSubscriber::builder()
            .with_max_level(Level::ERROR)
            .with_ansi(false)
            .with_target(false)
            .with_timer(ChronoLocal::rfc_3339())
            .with_writer(error_writer)
            .finish();
        if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
            eprintln!("logger already initialized or unavailable: {}", e);
        }
    }

    Ok(())
}
