use tauri::Manager;
use ntap_core::deps::{NPCAP_DIST_BASE_URL, NPCAP_INSTALLER_FILENAME};

pub async fn download_npcap(app_handle: &tauri::AppHandle) -> Result<u64, Box<dyn std::error::Error>> {
    let mut download_content_length: u64 = 0;
    let mut downloaded_bytes: u64 = 0;
    let npcap_installer_url = format!("{}{}", NPCAP_DIST_BASE_URL, NPCAP_INSTALLER_FILENAME);
    let download_dir = ntap_core::sys::get_download_dir_path().ok_or("Failed to get download dir")?;
    // Check and create download dir
    if !download_dir.exists() {
        std::fs::create_dir_all(&download_dir)?;
    }
    let npcap_target_path: std::path::PathBuf = download_dir.join(NPCAP_INSTALLER_FILENAME);
    // Download npcap installer if not exists
    if std::path::Path::new(&npcap_target_path).exists() {
        // return file size if file exists
        return Ok(std::fs::metadata(&npcap_target_path)?.len());
    }
    let installer_save_path: std::path::PathBuf = npcap_target_path.clone();
    // create a channel for progress
    let (progress_tx, mut progress_rx) = tokio::sync::mpsc::channel(100);
    // spawn a task to handle the progress
    tokio::spawn(async move {
        let _ = ntap_core::net::http::download_file_with_progress(npcap_installer_url, installer_save_path, progress_tx).await;
    });
    // Emit progress
    while let Some(progress) = progress_rx.recv().await {
        match progress {
            ntap_core::net::http::DownloadProgress::ContentLength(content_length) => {
                download_content_length = content_length; 
            }
            ntap_core::net::http::DownloadProgress::Downloaded(downloaded) => {
                downloaded_bytes = downloaded;
            }
        }
        match app_handle.emit_all("download_progress", progress) {
            Ok(_) => {}
            Err(e) => {
                log::error!("Error: {:?}", e);
            }
        }
    }
    if download_content_length == downloaded_bytes {
        Ok(download_content_length)
    }else{
        Err("Download failed".into())
    }
}

pub fn run_npcap_installer() -> Result<(), Box<dyn std::error::Error>> {
    let download_dir = ntap_core::sys::get_download_dir_path().ok_or("Failed to get download dir")?;
    let npcap_target_path: std::path::PathBuf = download_dir.join(NPCAP_INSTALLER_FILENAME);
    if !std::path::Path::new(&npcap_target_path).exists() {
        return Err("Npcap installer not found".into());
    }
    // Verify the checksum of the downloaded npcap installer
    match ntap_core::deps::verify_installer_checksum(&npcap_target_path) {
        Ok(_) => {},
        Err(e) => {
            return Err(e);
        },
    }
    // Run npcap installer
    match ntap_core::deps::run_npcap_installer(&npcap_target_path) {
        Ok(_) => {
            Ok(())
        }
        Err(e) => {
            Err(e)
        }
    }
}