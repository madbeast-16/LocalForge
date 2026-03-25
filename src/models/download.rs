use crate::error::{Result, LocalForgeError};
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;
use futures::StreamExt;

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

/// Resume a partial download if a `.part` file exists, using HTTP Range requests.
/// Falls back to a full download if the server doesn't support Range.
pub async fn download_file_resume<F>(
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

    // Already complete
    if dest.exists() {
        let meta = tokio::fs::metadata(&dest).await
            .map_err(|e| LocalForgeError::Io(e))?;
        on_progress(meta.len(), Some(meta.len()));
        return Ok(dest);
    }

    let tmp = dest.with_extension("part");

    // Check if partial file exists
    let (mut file, existing_size) = if tmp.exists() {
        let meta = tokio::fs::metadata(&tmp).await
            .map_err(|e| LocalForgeError::Io(e))?;
        let size = meta.len();
        let f = tokio::fs::OpenOptions::new()
            .append(true)
            .open(&tmp)
            .await
            .map_err(|e| LocalForgeError::Io(e))?;
        (f, size)
    } else {
        let f = tokio::fs::File::create(&tmp).await
            .map_err(|e| LocalForgeError::Io(e))?;
        (f, 0u64)
    };

    let mut downloaded = existing_size;

    // Build request with Range header if resuming
    let client = reqwest::Client::new();
    let mut req_builder = client.get(url);
    if existing_size > 0 {
        req_builder = req_builder.header("Range", format!("bytes={}-", existing_size));
    }

    let resp = req_builder.send().await
        .map_err(|e| LocalForgeError::DownloadFailed(e.to_string()))?;

    let status = resp.status();

    // 206 = Partial Content (resume successful)
    // 200 = Full content (server doesn't support Range, or fresh download)
    if status.as_u16() == 200 && existing_size > 0 {
        // Server doesn't support Range — restart from scratch
        drop(file);
        file = tokio::fs::File::create(&tmp).await
            .map_err(|e| LocalForgeError::Io(e))?;
        downloaded = 0;
    } else if !status.is_success() && status.as_u16() != 206 {
        return Err(LocalForgeError::DownloadFailed(
            format!("HTTP {}: {}", status, url),
        ));
    }

    let total: Option<u64> = resp.content_length().map(|cl| cl + existing_size);
    on_progress(downloaded, total);

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