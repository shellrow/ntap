use std::path::PathBuf;

use inquire::Confirm;
use ntap_core::deps::{NPCAP_DIST_BASE_URL, NPCAP_INSTALLER_FILENAME};

pub fn check_deps() -> Result<(), Box<dyn std::error::Error>> {
    match ntap_core::deps::check_deps() {
        Ok(_) => {
            return Ok(());
        }
        Err(e) => {
            match e {
                ntap_core::deps::DepsError::Missing(s) => {
                    if s == ntap_core::deps::NPCAP_SOFTWARE_NAME.to_string() {
                        let ans: bool = Confirm::new("Npcap is not installed, would you like to download & install it ?")
                            .prompt()
                            .unwrap();
                        if ans == false {
                            return Err("On windows, Npcap is required for ntap to work properly. Please install Npcap and try again.".into());
                        }
                        // Download the latest release of npcap installer
                        if let Some(download_dir) = ntap_core::sys::get_download_dir_path() {
                            let installer_path = download_npcap_with_progress(&download_dir)?;
                            println!("Npcap installer downloaded successfully: {}", installer_path.to_string_lossy());
                            // Install npcap
                            println!("Installing Npcap ...");
                            // Verify the checksum of the downloaded npcap installer
                            match ntap_core::deps::verify_installer_checksum(&installer_path) {
                                Ok(_) => println!("Npcap installer checksum is correct !"),
                                Err(e) => {
                                    println!("{}", e);
                                },
                            }
                            // Install npcap
                            match ntap_core::deps::run_npcap_installer(&installer_path) {
                                Ok(_) => println!("Npcap installed successfully !"),
                                Err(e) => {
                                    println!("{}", e);
                                },
                            }
                            println!("Npcap installed successfully.");
                        }
                    }
                }
                ntap_core::deps::DepsError::Unknown(s) => {
                    eprintln!("Error: Unknown dependency: {}", s);
                }
            }
        }
    }
    Ok(())
}

/// Download npcap installer with progress
pub fn download_npcap_with_progress(dst_dir_path: &PathBuf) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let npcap_installer_url = format!("{}{}", NPCAP_DIST_BASE_URL, NPCAP_INSTALLER_FILENAME);
    // Check and create download dir
    if !dst_dir_path.exists() {
        std::fs::create_dir_all(&dst_dir_path)?;
    }
    let npcap_target_path: std::path::PathBuf = dst_dir_path.join(NPCAP_INSTALLER_FILENAME);
    // Download npcap installer if not exists
    if std::path::Path::new(&npcap_target_path).exists() {
        return Ok(npcap_target_path); 
    }
    let rt = tokio::runtime::Runtime::new().unwrap();
    let installer_save_path: PathBuf = npcap_target_path.clone();
    rt.block_on(async {
        // create a channel for progress
        let (progress_tx, mut progress_rx) = tokio::sync::mpsc::channel(100);
        // spawn a task to handle the progress
        tokio::spawn(async move {
            let _ = ntap_core::net::http::download_file_with_progress(npcap_installer_url, installer_save_path, progress_tx).await;
        });
        // Display progress with indicatif
        let bar = indicatif::ProgressBar::new(1000);
        bar.set_style(indicatif::ProgressStyle::default_bar().template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})").progress_chars("#>-"));
        while let Some(progress) = progress_rx.recv().await {
            match progress {
                ntap_core::net::http::DownloadProgress::ContentLength(content_length) => {
                    println!("Content-Length: {}", content_length);
                    bar.set_length(content_length);
                }
                ntap_core::net::http::DownloadProgress::Downloaded(downloaded) => {
                    bar.set_position(downloaded);
                }
            }
        }
        bar.finish();
    });
    Ok(npcap_target_path)
}
