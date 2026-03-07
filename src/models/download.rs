use anyhow::{Context, Result};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Download a file from `url` into `dest_dir`, returning the final path.
/// Reports progress via the `on_progress` callback: (bytes_downloaded, total_bytes).
pub fn download_file<F>(
    url: &str,
    dest_dir: &Path,
    filename: &str,
    mut on_progress: F,
) -> Result<PathBuf>
where
    F: FnMut(u64, Option<u64>),
{
    std::fs::create_dir_all(dest_dir)?;
    let dest = dest_dir.join(filename);

    if dest.exists() {
        let meta = std::fs::metadata(&dest)?;
        on_progress(meta.len(), Some(meta.len()));
        return Ok(dest);
    }

    let resp = ureq::get(url)
        .call()
        .context("Failed to initiate download")?;

    let total: Option<u64> = resp.header("Content-Length").and_then(|v| v.parse().ok());

    let mut reader = resp.into_reader();
    let tmp = dest.with_extension("part");
    let mut file = std::fs::File::create(&tmp)?;

    let mut downloaded: u64 = 0;
    let mut buf = [0u8; 65536];

    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n])?;
        downloaded += n as u64;
        on_progress(downloaded, total);
    }

    file.flush()?;
    drop(file);
    std::fs::rename(&tmp, &dest)?;

    Ok(dest)
}

/// Convenience: download with no progress callback.
pub fn download_file_simple(url: &str, dest_dir: &Path, filename: &str) -> Result<PathBuf> {
    download_file(url, dest_dir, filename, |_, _| {})
}
