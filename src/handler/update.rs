use anyhow::Result;
use tracing::Level;
use tracing_subscriber::{fmt::time::ChronoLocal, FmtSubscriber};
use std::path::PathBuf;

fn download_file(
    url: &str,
    save_dir_path: PathBuf,
    file_name: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Check and create download dir
    if !save_dir_path.exists() {
        std::fs::create_dir_all(&save_dir_path)?;
    }
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            return Err(Box::new(e));
        }
    };
    let save_file_path: PathBuf = save_dir_path.join(file_name);
    rt.block_on(async {
        tracing::info!("Downloading {} from {}", file_name, url);
        // create a channel for progress
        let (progress_tx, mut progress_rx) = tokio::sync::mpsc::channel(100);
        let file_url: String = url.to_string();
        let file_path: PathBuf = save_file_path.clone();
        // spawn a task to handle the progress
        tokio::spawn(async move {
            let _ = crate::net::http::download_file_with_progress(file_url, file_path, progress_tx).await;
        });
        // Display progress with indicatif
        let bar = indicatif::ProgressBar::new(1000);
        bar.set_style(indicatif::ProgressStyle::default_bar().template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})").unwrap().progress_chars("#>-"));
        while let Some(progress) = progress_rx.recv().await {
            match progress {
                crate::net::http::DownloadProgress::ContentLength(content_length) => {
                    tracing::info!("File URL: {}, Content-Length: {}", url, content_length);
                    bar.set_length(content_length);
                }
                crate::net::http::DownloadProgress::Downloaded(downloaded) => {
                    bar.set_position(downloaded);
                }
            }
        }
        bar.finish();
        tracing::info!("Downloaded {} to {}", file_name, save_file_path.display());
    });
    Ok(save_file_path)
}

pub fn download_db_files() -> Result<()> {
    // Init logger
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_timer(ChronoLocal::rfc_3339())
        .finish();
    if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
        eprintln!("logger already initialized or unavailable: {}", e);
    }

    let database_dir = crate::sys::get_database_dir_path()
        .ok_or_else(|| anyhow::anyhow!("failed to resolve database directory path"))?;
    // OUI
    match download_file(
        crate::db::oui::OUI_R2_URL,
        database_dir.clone(),
        crate::db::oui::OUI_CSV_NAME,
    ) {
        Ok(_) => {}
        Err(e) => {
            tracing::error!("{:?}", e);
        }
    }
    // TCP Service
    match download_file(
        crate::db::service::TCP_SERVICE_R2_URL,
        database_dir.clone(),
        crate::db::service::TCP_SERVICE_CSV_NAME,
    ) {
        Ok(_) => {}
        Err(e) => {
            tracing::error!("{:?}", e);
        }
    }
    // UDP Service
    match download_file(
        crate::db::service::UDP_SERVICE_R2_URL,
        database_dir,
        crate::db::service::UDP_SERVICE_CSV_NAME,
    ) {
        Ok(_) => {}
        Err(e) => {
            tracing::error!("{:?}", e);
        }
    }
    tracing::info!("Successfully downloaded ntap databases.");
    Ok(())
}
