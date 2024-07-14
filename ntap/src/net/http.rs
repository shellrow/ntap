use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use std::{error::Error, io::Write, path::PathBuf};

pub const DEFAULT_USER_AGENT_FIREFOX: &str =
    "Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/121.0";
pub const DEFAULT_USER_AGENT_CHROME: &str =
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DownloadProgress {
    ContentLength(u64),
    Downloaded(u64),
}

pub async fn download_file(url: String, save_file_path: PathBuf) -> Result<(), Box<dyn Error>> {
    // Check and create download dir
    let file_path = std::path::Path::new(&save_file_path);
    if let Some(parent_dir) = file_path.parent() {
        if !parent_dir.exists() {
            std::fs::create_dir_all(parent_dir)?;
        }
    } else {
        return Err("Invalid save file path".into());
    }
    // Download file
    let response = reqwest::get(&url).await?;
    let mut dest = std::fs::File::create(&save_file_path)?;
    let content = response.bytes().await?;
    dest.write_all(&content)?;
    Ok(())
}

pub async fn download_file_with_progress(
    url: String,
    save_file_path: PathBuf,
    progress_tx: tokio::sync::mpsc::Sender<DownloadProgress>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check and create download dir
    let file_path = std::path::Path::new(&save_file_path);
    if let Some(parent_dir) = file_path.parent() {
        if !parent_dir.exists() {
            std::fs::create_dir_all(parent_dir)?;
        }
    } else {
        return Err("Invalid save file path".into());
    }
    // Download file with progress
    let response = reqwest::get(&url).await?;
    let content_length = response.content_length().unwrap_or(0);
    progress_tx
        .send(DownloadProgress::ContentLength(content_length))
        .await?;
    let mut dest = std::fs::File::create(&save_file_path)?;
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        downloaded += chunk.len() as u64;
        dest.write_all(&chunk)?;
        progress_tx
            .send(DownloadProgress::Downloaded(downloaded))
            .await?;
    }
    Ok(())
}
