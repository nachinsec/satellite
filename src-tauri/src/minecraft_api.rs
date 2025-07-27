use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::io::Write;
use std::path::Path;
use tauri::Emitter;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LauncherError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),

    #[error("JSON parsing error: {0}")]
    JsonParsing(#[from] serde_json::Error),

    #[error("Version not found: {version}")]
    VersionNotFound { version: String },

    #[error("Failed to launch Minecraft: {error}")]
    MinecraftLaunchError { error: String },

    #[error("Configuration validation error in {field}: {message}")]
    ConfigValidation { field: String, message: String },
}

pub type Result<T> = std::result::Result<T, LauncherError>;

#[derive(Debug, Deserialize)]
pub struct VersionManifest {
    pub versions: Vec<MinecraftVersion>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MinecraftVersion {
    pub id: String,
    pub r#type: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionJson {
    pub main_class: String,
    pub arguments: Option<Arguments>,
    pub libraries: Vec<Library>,
    pub downloads: Downloads,
    pub asset_index: AssetIndex,
}

#[derive(Debug, Deserialize)]
pub struct Arguments {
    pub game: Option<Vec<Value>>,
    pub jvm: Option<Vec<Value>>,
}

#[derive(Debug, Deserialize)]
pub struct Library {
    pub name: String,
    pub downloads: Option<LibraryDownloads>,
}

#[derive(Debug, Deserialize)]
pub struct LibraryDownloads {
    pub artifact: Option<DownloadInfo>,
    pub classifiers: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct DownloadInfo {
    pub url: String,
    pub path: Option<String>,
    pub sha1: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Downloads {
    pub client: DownloadInfo,
}

#[derive(Debug, Deserialize)]
pub struct AssetIndex {
    pub id: String,
    pub url: String,
}
pub async fn download_version_manifest(client: &Client) -> Result<String> {
    let url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
    let resp = client.get(url).send().await?;
    let text = resp.text().await?;
    Ok(text)
}

pub fn parse_version_manifest(json: &str) -> Result<VersionManifest> {
    let manifest: VersionManifest = serde_json::from_str(json)?;
    Ok(manifest)
}

pub async fn download_version_json(client: &Client, url: &str) -> Result<String> {
    let resp = client.get(url).send().await?;
    let text = resp.text().await?;
    Ok(text)
}

pub fn parse_version_json(json: &str) -> Result<VersionJson> {
    let version_json: VersionJson = serde_json::from_str(json)?;
    Ok(version_json)
}

pub async fn download_file(client: &Client, url: &str, path: &str) -> Result<()> {
    let resp = client.get(url).send().await?;
    let bytes = resp.bytes().await?;
    let parent = Path::new(path).parent().unwrap();
    if !parent.exists() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::File::create(path)?;
    file.write_all(&bytes)?;
    Ok(())
}

pub async fn download_assets(
    client: &reqwest::Client,
    assets_index_path: &str,
    base_dir: &str,
    window: tauri::Window,
) -> Result<()> {
    use futures::stream::{FuturesUnordered, StreamExt};
    use serde_json::Value;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    let index_str = std::fs::read_to_string(assets_index_path)?;
    let index_json: Value = serde_json::from_str(&index_str)?;
    let objects = &index_json["objects"];
    let total = objects.as_object().unwrap().len();
    let progress = Arc::new(Mutex::new(0));
    let mut futures = FuturesUnordered::new();

    for (asset_name, asset_info) in objects.as_object().unwrap() {
        let hash = asset_info["hash"].as_str().unwrap().to_owned();
        let subdir = hash[0..2].to_string();
        let asset_url = format!(
            "https://resources.download.minecraft.net/{}/{}",
            subdir, hash
        );
        let asset_path = format!("{}/assets/objects/{}/{}", base_dir, subdir, hash);

        if !std::path::Path::new(&asset_path).exists() {
            let client = client.clone();
            let window = window.clone();
            let asset_name = asset_name.clone();
            let progress = Arc::clone(&progress);
            futures.push(tokio::spawn(async move {
                let _ = download_file(&client, &asset_url, &asset_path).await;
                let mut prog = progress.lock().await;
                *prog += 1;
                let _ = window.emit("progress", *prog as f64 / total as f64);
                let _ = window.emit("log", format!("Downloaded asset: {}", asset_name));
                Ok::<(), LauncherError>(())
            }));
        }
    }

    while let Some(result) = futures.next().await {
        match result {
            Ok(_) => {}
            Err(e) => {
                let _ = window
                    .emit("error", format!("Error downloading assets: {}", e))
                    .ok();
            }
        }
    }
    let _ = window.emit("progress", 1.0);
    Ok(())
}
