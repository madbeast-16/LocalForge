use crate::error::{Result, LocalForgeError};
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;
use futures::StreamExt;
use futures::stream::TryStreamExt;

/// Download a file from `url` into `dest_dir`, returning the final path.
/// Reports progress via the `on_progress` callback: (bytes_downloaded, total_bytes).
pub async fn download_file<F>(
    url: &str,
    dest_dir: &Path,
    filename: &str,
    mut on_progress: F,
) -> Result<PathBuf>
where
    F: FnMut(u64, Option<u64>) + Send,
{
    tokio::fs::create_dir_all(dest_dir).await
        .map_err(|e| LocalForgeError::Io(e))?;
    let dest = dest_dir.join(filename);

    if dest.exists() {
        let meta = tokio::fs::metadata(&dest).await
            .map_err(|e| LocalForgeError::Io(e))?;
        on_progress(meta.len(), Some(meta.len()));
        return Ok(dest);
    }

    let client = reqwest::Client::new();
    let resp = client.get(url).send().await
        .map_err(|e| LocalForgeError::DownloadFailed(e.to_string()))?;
    
    if !resp.status().is_success() {
        return Err(LocalForgeError::DownloadFailed(format!("HTTP {}: {}", resp.status(), url)));
    }

    let total: Option<u64> = resp.content_length();

    let tmp = dest.with_extension("part");
    let mut file = tokio::fs::File::create(&tmp).await
        .map_err(|e| LocalForgeError::Io(e))?;

    let mut downloaded: u64 = 0;
    let mut stream = resp.bytes_stream();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result
            .map_err(|e: reqwest::Error| LocalForgeError::Network(e.to_string()))?;
        
        file.write_all(&chunk).await
            .map_err(|e: std::io::Error| LocalForgeError::Io(e))?;
        
        downloaded += chunk.len() as u64;
        on_progress(downloaded, total);
    }

    file.flush().await
        .map_err(|e| LocalForgeError::Io(e))?;
    drop(file);
    
    tokio::fs::rename(&tmp, &dest).await
        .map_err(|e| LocalForgeError::Io(e))?;

    Ok(dest)
}

/// Convenience: download with no progress callback.
pub async fn download_file_simple(url: &str, dest_dir: &Path, filename: &str) -> Result<PathBuf> {
    download_file(url, dest_dir, filename, |_, _| {}).await
}

/// Resume a partial download if a `.part` file exists.
pub async fn download_file_resume<F>(
    url: &str,
    dest_dir: &Path,
    filename: &str,
    on_progress: F,
) -> Result<PathBuf>
where
    F: FnMut(u64, Option<u64>) + Send,
{
    // For now, just do a fresh download
    // TODO: implement proper range requests for resume
    download_file(url, dest_dir, filename, on_progress).await
}