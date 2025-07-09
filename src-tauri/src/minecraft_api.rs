use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct VersionManifest {
    pub versions: Vec<MinecraftVersion>,
}

#[derive(Debug, Deserialize)]
pub struct MinecraftVersion {
    pub id: String,
    pub r#type: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct VersionJson {
    pub mainClass: String,
    pub arguments: Option<Arguments>,
    pub libraries: Vec<Library>,
    pub downloads: Downloads,
    pub assetIndex: AssetIndex,
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
pub fn download_version_manifest() -> Result<String, Box<dyn std::error::Error>> {
    let url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
    let resp = reqwest::blocking::get(url)?;
    let text = resp.text()?;
    Ok(text)
}

pub fn parse_version_manifest(json: &str) -> Result<VersionManifest, Box<dyn std::error::Error>> {
    let manifest: VersionManifest = serde_json::from_str(json)?;
    Ok(manifest)
}

pub fn download_version_json(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let resp = reqwest::blocking::get(url)?;
    let text = resp.text()?;
    Ok(text)
}

pub fn parse_version_json(json: &str) -> Result<VersionJson, Box<dyn std::error::Error>> {
    let version_json: VersionJson = serde_json::from_str(json)?;
    Ok(version_json)
}

pub fn download_file(url: &str, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let resp = reqwest::blocking::get(url)?;
    let bytes = resp.bytes()?;
    let parent = Path::new(path).parent().unwrap();
    if !parent.exists() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::File::create(path)?;
    file.write_all(&bytes)?;
    Ok(())
}

pub fn download_assets(assets_index_path: &str, base_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let index_str = std::fs::read_to_string(assets_index_path)?;
    let index_json: Value = serde_json::from_str(&index_str)?;
    let objects = &index_json["objects"];

    for (asset_name, asset_info) in objects.as_object().unwrap() {
        let hash = asset_info["hash"].as_str().unwrap();
        let subdir = &hash[0..2];
        let asset_url = format!("https://resources.download.minecraft.net/{}/{}", subdir, hash);
        let asset_path = format!("{}/assets/objects/{}/{}", base_dir, subdir, hash);
        if !std::path::Path::new(&asset_path).exists() {
            download_file(&asset_url, &asset_path)?;
            println!("Descargado asset: {}", asset_name);
        }
    }
    Ok(())
}