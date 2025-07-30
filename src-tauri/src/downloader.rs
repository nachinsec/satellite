use crate::errors::{LauncherError, LauncherResult};
use reqwest::Client;
use sha1::{Digest, Sha1};
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use tauri::Emitter;

pub struct Downloader {
    client: Client,
}

impl Downloader {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Download a file with progress reporting
    pub async fn download_file_with_progress(
        &self,
        url: &str,
        path: &str,
        window: Option<&tauri::Window>,
    ) -> LauncherResult<()> {
        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent).map_err(|e| LauncherError::Io(e))?;
        }

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| LauncherError::DownloadFailed {
                url: url.to_string(),
                reason: e.to_string(),
            })?;

        if !response.status().is_success() {
            return Err(LauncherError::DownloadFailed {
                url: url.to_string(),
                reason: format!("HTTP {}", response.status()),
            });
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| LauncherError::DownloadFailed {
                url: url.to_string(),
                reason: e.to_string(),
            })?;

        fs::write(path, bytes).map_err(|e| LauncherError::Io(e))?;

        if let Some(window) = window {
            window
                .emit("log", format!("Downloaded: {}", path))
                .ok();
        }

        Ok(())
    }

    /// Verify file hash and re-download if necessary
    pub async fn verify_and_download(
        &self,
        url: &str,
        path: &str,
        expected_sha1: Option<&str>,
        window: Option<&tauri::Window>,
    ) -> LauncherResult<bool> {
        let mut needs_download = true;

        // Check if file exists and verify hash if provided
        if Path::new(path).exists() {
            if let Some(expected_hash) = expected_sha1 {
                match self.calculate_file_hash(path) {
                    Ok(actual_hash) => {
                        if actual_hash == expected_hash {
                            needs_download = false;
                            if let Some(window) = window {
                                window
                                    .emit("log", format!("File OK: {}", path))
                                    .ok();
                            }
                        } else {
                            if let Some(window) = window {
                                window
                                    .emit("log", format!("Hash mismatch, re-downloading: {}", path))
                                    .ok();
                            }
                        }
                    }
                    Err(_) => {
                        // If we can't calculate hash, re-download
                        needs_download = true;
                    }
                }
            } else {
                // No hash provided, assume file is OK if it exists
                needs_download = false;
            }
        }

        if needs_download {
            self.download_file_with_progress(url, path, window).await?;
        }

        Ok(needs_download)
    }

    /// Calculate SHA1 hash of a file
    pub fn calculate_file_hash(&self, path: &str) -> LauncherResult<String> {
        let mut file = File::open(path).map_err(|e| LauncherError::Io(e))?;
        let mut hasher = Sha1::new();
        let mut buffer = [0; 8192];

        loop {
            let n = file.read(&mut buffer).map_err(|e| LauncherError::Io(e))?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Download multiple files in parallel
    pub async fn download_parallel<I, F>(
        &self,
        downloads: I,
        window: Option<&tauri::Window>,
    ) -> LauncherResult<()>
    where
        I: IntoIterator<Item = F>,
        F: std::future::Future<Output = LauncherResult<()>> + Send,
    {
        use futures::future::try_join_all;

        let futures: Vec<_> = downloads.into_iter().collect();
        
        if let Some(window) = window {
            window
                .emit("log", format!("Starting {} parallel downloads...", futures.len()))
                .ok();
        }

        try_join_all(futures).await?;

        if let Some(window) = window {
            window
                .emit("log", "All parallel downloads completed!")
                .ok();
        }

        Ok(())
    }
}
