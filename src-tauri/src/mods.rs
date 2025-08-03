use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use crate::errors::LauncherError;

// Modrinth API response structures
#[derive(Debug, Deserialize)]
struct ModrinthSearchResponse {
    hits: Vec<ModrinthSearchHit>,
}

#[derive(Debug, Deserialize)]
struct ModrinthSearchHit {
    slug: String,
    title: String,
    description: String,
    categories: Vec<String>,
    versions: Vec<String>,
    downloads: u64,
    icon_url: Option<String>,
    author: String,
    project_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub minecraft_version: String,
    pub mod_loader: ModLoader,
    pub file_name: String,
    pub file_size: u64,
    pub enabled: bool,
    pub dependencies: Vec<String>,
    pub source: ModSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModLoader {
    Forge,
    Fabric,
    Quilt,
    NeoForge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModSource {
    Local,
    Modrinth { project_id: String },
    CurseForge { project_id: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModSearchResult {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub downloads: u64,
    pub icon_url: Option<String>,
    pub minecraft_versions: Vec<String>,
    pub mod_loaders: Vec<ModLoader>,
    pub source: ModSource,
}

pub struct ModManager {
    mods_directory: PathBuf,
}

impl ModManager {
    pub fn new(game_directory: &str) -> Self {
        let mods_directory = Path::new(game_directory).join("mods");
        Self { mods_directory }
    }

    /// Ensure mods directory exists
    pub fn ensure_mods_directory(&self) -> Result<(), LauncherError> {
        if !self.mods_directory.exists() {
            fs::create_dir_all(&self.mods_directory)?;
        }
        Ok(())
    }

    /// Get all installed mods
    pub fn get_installed_mods(&self) -> Result<Vec<ModInfo>, LauncherError> {
        self.ensure_mods_directory()?;
        
        let mut mods = Vec::new();
        
        if let Ok(entries) = fs::read_dir(&self.mods_directory) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("jar") {
                    if let Ok(mod_info) = self.parse_mod_file(&path) {
                        mods.push(mod_info);
                    }
                }
            }
        }
        
        Ok(mods)
    }

    /// Parse mod information from jar file
    fn parse_mod_file(&self, path: &Path) -> Result<ModInfo, LauncherError> {
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown.jar")
            .to_string();
            
        let metadata = fs::metadata(path)?;
        let file_size = metadata.len();
        
        // For now, create basic mod info from filename
        // In a real implementation, you'd parse the mod's metadata from the JAR
        let mod_info = ModInfo {
            id: self.generate_mod_id(&file_name),
            name: self.extract_mod_name(&file_name),
            version: "unknown".to_string(),
            description: None,
            author: None,
            minecraft_version: "1.20.1".to_string(), // Default, should be parsed
            mod_loader: ModLoader::Fabric, // Default, should be detected
            file_name,
            file_size,
            enabled: !path.extension().map_or(false, |ext| ext == "disabled"),
            dependencies: Vec::new(),
            source: ModSource::Local,
        };
        
        Ok(mod_info)
    }

    /// Generate a unique ID for a mod based on filename
    fn generate_mod_id(&self, file_name: &str) -> String {
        file_name.replace(".jar", "").replace(" ", "_").to_lowercase()
    }

    /// Extract mod name from filename
    fn extract_mod_name(&self, file_name: &str) -> String {
        file_name
            .replace(".jar", "")
            .replace("_", " ")
            .replace("-", " ")
    }

    /// Enable/disable a mod
    pub fn toggle_mod(&self, mod_id: &str, enabled: bool) -> Result<(), LauncherError> {
        let mods = self.get_installed_mods()?;
        
        if let Some(mod_info) = mods.iter().find(|m| m.id == mod_id) {
            let current_path = self.mods_directory.join(&mod_info.file_name);
            
            if enabled && mod_info.file_name.ends_with(".disabled") {
                // Enable mod (remove .disabled extension)
                let new_name = mod_info.file_name.replace(".disabled", "");
                let new_path = self.mods_directory.join(new_name);
                fs::rename(current_path, new_path)?;
            } else if !enabled && !mod_info.file_name.ends_with(".disabled") {
                // Disable mod (add .disabled extension)
                let new_name = format!("{}.disabled", mod_info.file_name);
                let new_path = self.mods_directory.join(new_name);
                fs::rename(current_path, new_path)?;
            }
        }
        
        Ok(())
    }

    /// Delete a mod
    pub fn delete_mod(&self, mod_id: &str) -> Result<(), LauncherError> {
        let mods = self.get_installed_mods()?;
        
        if let Some(mod_info) = mods.iter().find(|m| m.id == mod_id) {
            let mod_path = self.mods_directory.join(&mod_info.file_name);
            fs::remove_file(mod_path)?;
        }
        
        Ok(())
    }

    /// Install a mod from a file path
    pub fn install_mod_from_file(&self, source_path: &str) -> Result<ModInfo, LauncherError> {
        self.ensure_mods_directory()?;
        
        let source = Path::new(source_path);
        if !source.exists() {
            return Err(LauncherError::FileNotFound(source_path.to_string()));
        }

        let file_name = source.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| LauncherError::InvalidPath("Invalid file name".to_string()))?;

        let destination = self.mods_directory.join(file_name);
        fs::copy(source, &destination)?;
        
        self.parse_mod_file(&destination)
    }

    /// Install a mod from online source (Modrinth)
    pub async fn install_mod_online<F>(
        &self,
        mod_id: &str,
        minecraft_version: &str,
        mod_loader: &ModLoader,
        progress_callback: F,
    ) -> Result<ModInfo, LauncherError>
    where
        F: Fn(u8),
    {
        self.ensure_mods_directory()?;
        
        // Get project info from Modrinth
        let client = reqwest::Client::new();
        
        progress_callback(10);
        
        // First, get the project to get the correct project ID
        let project_url = format!("https://api.modrinth.com/v2/project/{}", mod_id);
        let project_response = client
            .get(&project_url)
            .header("User-Agent", "Satellite-Launcher/0.3.0 (contact@satellite-launcher.com)")
            .send()
            .await?;
            
        if !project_response.status().is_success() {
            return Err(LauncherError::DownloadFailed {
                url: project_url,
                reason: format!("Failed to get project info: {}", project_response.status()),
            });
        }
        
        let project: serde_json::Value = project_response.json().await?;
        let project_id = project["id"].as_str().unwrap_or(mod_id);
        
        progress_callback(20);
        
        // Get versions for this project
        let versions_url = format!("https://api.modrinth.com/v2/project/{}/version", project_id);
        let versions_response = client
            .get(&versions_url)
            .header("User-Agent", "Satellite-Launcher/0.3.0 (contact@satellite-launcher.com)")
            .send()
            .await?;
            
        if !versions_response.status().is_success() {
            return Err(LauncherError::DownloadFailed {
                url: versions_url,
                reason: format!("Failed to get versions: {}", versions_response.status()),
            });
        }
        
        let versions: Vec<serde_json::Value> = versions_response.json().await?;
        
        progress_callback(30);
        
        // Find the best matching version
        let loader_str = match mod_loader {
            ModLoader::Fabric => "fabric",
            ModLoader::Forge => "forge",
            ModLoader::Quilt => "quilt",
            ModLoader::NeoForge => "neoforge",
        };
        
        let mut best_version = None;
        for version in &versions {
            let empty_vec = vec![];
            let game_versions = version["game_versions"].as_array().unwrap_or(&empty_vec);
            let loaders = version["loaders"].as_array().unwrap_or(&empty_vec);
            
            let supports_version = game_versions.iter().any(|v| 
                v.as_str().map(|s| s == minecraft_version).unwrap_or(false)
            );
            let supports_loader = loaders.iter().any(|l| 
                l.as_str().map(|s| s == loader_str).unwrap_or(false)
            );
            
            if supports_version && supports_loader {
                best_version = Some(version);
                break;
            }
        }
        
        let version = best_version.ok_or_else(|| LauncherError::DownloadFailed {
            url: versions_url.clone(),
            reason: format!("No compatible version found for Minecraft {} with {}", minecraft_version, loader_str),
        })?;
        
        progress_callback(40);
        
        // Get the download file
        let files = version["files"].as_array().ok_or_else(|| LauncherError::DownloadFailed {
            url: versions_url.clone(),
            reason: "No files found in version".to_string(),
        })?;
        
        let primary_file = files.iter().find(|f| 
            f["primary"].as_bool().unwrap_or(false)
        ).or_else(|| files.first()).ok_or_else(|| LauncherError::DownloadFailed {
            url: versions_url.clone(),
            reason: "No downloadable files found".to_string(),
        })?;
        
        let download_url = primary_file["url"].as_str().ok_or_else(|| LauncherError::DownloadFailed {
            url: versions_url.clone(),
            reason: "No download URL found".to_string(),
        })?;
        
        let filename = primary_file["filename"].as_str().ok_or_else(|| LauncherError::DownloadFailed {
            url: versions_url.clone(),
            reason: "No filename found".to_string(),
        })?;
        
        progress_callback(50);
        
        // Download the file
        let download_response = client
            .get(download_url)
            .header("User-Agent", "Satellite-Launcher/0.3.0 (contact@satellite-launcher.com)")
            .send()
            .await?;
            
        if !download_response.status().is_success() {
            return Err(LauncherError::DownloadFailed {
                url: download_url.to_string(),
                reason: format!("Download failed: {}", download_response.status()),
            });
        }
        
        progress_callback(70);
        
        // Save the file
        let destination = self.mods_directory.join(filename);
        let bytes = download_response.bytes().await?;
        fs::write(&destination, bytes)?;
        
        progress_callback(90);
        
        // Parse the mod file to get mod info
        let mod_info = self.parse_mod_file(&destination)?;
        
        progress_callback(100);
        
        Ok(mod_info)
    }

}

/// Search for mods on Modrinth using the real API
pub async fn search_mods(
    query: &str,
    minecraft_version: &str,
    mod_loader: &ModLoader,
    limit: u32,
) -> Result<Vec<ModSearchResult>, LauncherError> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    let client = reqwest::Client::new();
    
    // Convert our ModLoader enum to Modrinth's format
    let loader_filter = match mod_loader {
        ModLoader::Fabric => "fabric",
        ModLoader::Forge => "forge",
        ModLoader::Quilt => "quilt",
        ModLoader::NeoForge => "neoforge",
    };
    
    // Build the search URL with more flexible filters
    // Only filter by project type (mod) and include version/loader as optional
    let url = format!(
        "https://api.modrinth.com/v2/search?query={}&limit={}&facets=[[\"project_type:mod\"], [\"versions:{}\"], [\"categories:{}\"]]",
        urlencoding::encode(query),
        limit,
        minecraft_version,
        loader_filter
    );
    
    println!("Searching Modrinth with URL: {}", url); // Debug log
    println!("Filters: minecraft_version={}, mod_loader={}", minecraft_version, loader_filter);
    
    let response = client
        .get(&url)
        .header("User-Agent", "Satellite-Launcher/0.3.0 (contact@satellite-launcher.com)")
        .send()
        .await?;
    
    if !response.status().is_success() {
        return Err(LauncherError::DownloadFailed {
            url: url.clone(),
            reason: format!("Modrinth API returned status: {}", response.status()),
        });
    }
    
    let search_response: ModrinthSearchResponse = response.json().await?;
    
    let mut results = Vec::new();
    for hit in search_response.hits {
        let mod_result = ModSearchResult {
            id: hit.slug.clone(),
            name: hit.title,
            description: hit.description,
            author: hit.author,
            downloads: hit.downloads,
            icon_url: hit.icon_url,
            minecraft_versions: hit.versions,
            mod_loaders: hit.categories
                .iter()
                .filter_map(|cat| match cat.as_str() {
                    "fabric" => Some(ModLoader::Fabric),
                    "forge" => Some(ModLoader::Forge),
                    "quilt" => Some(ModLoader::Quilt),
                    "neoforge" => Some(ModLoader::NeoForge),
                    _ => None,
                })
                .collect(),
            source: ModSource::Modrinth { project_id: hit.project_id },
        };
        results.push(mod_result);
    }
    
    Ok(results)
}
