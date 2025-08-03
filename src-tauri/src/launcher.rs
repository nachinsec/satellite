use crate::config::LauncherConfig;
use crate::downloader::Downloader;
use crate::errors::{LauncherError, LauncherResult};
use crate::minecraft_api::{self, MinecraftVersion};
use std::process::Command;
use tauri::Emitter;
use crate::mods::ModManager;

pub struct MinecraftLauncher {
    downloader: Downloader,
}

impl MinecraftLauncher {
    pub fn new() -> Self {
        Self {
            downloader: Downloader::new(),
        }
    }

    /// Launch Minecraft with the specified version
    pub async fn launch_version(
        &self,
        window: tauri::Window,
        version: String,
    ) -> LauncherResult<()> {
        let config = LauncherConfig::load().map_err(|e| LauncherError::ConfigValidation {
            field: "config".to_string(),
            message: e.to_string(),
        })?;

        let base_dir = &config.game_directory;

        // Check if mods are installed to determine if we need Fabric
        let mod_manager = ModManager::new(base_dir);
        let installed_mods = mod_manager.get_installed_mods().unwrap_or_default();
        let needs_fabric = !installed_mods.is_empty();

        if needs_fabric {
            window.emit("log", format!("🔧 {} mods detected! Setting up Fabric automatically...", installed_mods.len())).ok();
            self.ensure_fabric_installed(&version, base_dir, &window).await?;
            self.launch_with_fabric(&config, &version, base_dir, &window).await?;
        } else {
            window.emit("log", "🎮 No mods detected, launching vanilla Minecraft...").ok();
            self.launch_vanilla(&config, &version, base_dir, &window).await?;
        }

        window
            .emit("log", "🚀 Minecraft launched successfully!")
            .ok();

        Ok(())
    }

    /// Download all required libraries
    async fn download_libraries(
        &self,
        libraries: &[minecraft_api::Library],
        base_dir: &str,
        window: &tauri::Window,
    ) -> LauncherResult<()> {
        for lib in libraries {
            if let Some(downloads) = &lib.downloads {
                if let Some(artifact) = &downloads.artifact {
                    if let Some(lib_path_rel) = &artifact.path {
                        let lib_path = format!("{}/libraries/{}", base_dir, lib_path_rel);
                        let expected_sha1 = artifact.sha1.as_deref();
                        
                        self.downloader
                            .verify_and_download(&artifact.url, &lib_path, expected_sha1, Some(window))
                            .await?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Download game assets
    async fn download_assets(
        &self,
        asset_index: &minecraft_api::AssetIndex,
        base_dir: &str,
        window: &tauri::Window,
    ) -> LauncherResult<()> {
        let assets_index_path = format!("{}/assets/indexes/{}.json", base_dir, asset_index.id);
        
        // Download asset index
        self.downloader
            .download_file_with_progress(&asset_index.url, &assets_index_path, Some(window))
            .await?;

        // Download individual assets
        let asset_futures = minecraft_api::download_assets(
            self.downloader.client(),
            &assets_index_path,
            base_dir,
            window.clone(),
        );
        
        asset_futures.await?;
        Ok(())
    }

    /// Launch the Minecraft process
    async fn launch_minecraft(
        &self,
        config: &LauncherConfig,
        version_json: &minecraft_api::VersionJson,
        selected_version: &MinecraftVersion,
        jar_path: &str,
        base_dir: &str,
    ) -> LauncherResult<()> {
        // Build classpath
        let mut classpath = Vec::new();
        
        for lib in &version_json.libraries {
            if let Some(downloads) = &lib.downloads {
                if let Some(artifact) = &downloads.artifact {
                    if let Some(lib_path_rel) = &artifact.path {
                        let lib_path = format!("{}/libraries/{}", base_dir, lib_path_rel);
                        classpath.push(lib_path);
                    }
                }
            }
        }
        
        classpath.push(jar_path.to_string());
        let classpath_str = classpath.join(";");

        // Prepare Java command
        let java_executable = config.get_java_executable();
        let java_args = config.get_jvm_args();
        
        let mut command = Command::new(&java_executable);
        
        // Add JVM arguments
        for arg in java_args {
            command.arg(arg);
        }

        // Add Minecraft arguments
        let _output = command
            .arg("-cp")
            .arg(&classpath_str)
            .arg(&version_json.main_class)
            .arg("--username")
            .arg(&config.player_name)
            .arg("--version")
            .arg(&selected_version.id)
            .arg("--gameDir")
            .arg(base_dir)
            .arg("--assetsDir")
            .arg(format!("{}/assets", base_dir))
            .arg("--assetIndex")
            .arg(&version_json.asset_index.id)
            .arg("--uuid")
            .arg(&config.player_uuid.clone().unwrap_or_default())
            .arg("--accessToken")
            .arg("N/A")
            .arg("--userType")
            .arg("legacy")
            .arg("--versionType")
            .arg(&selected_version.r#type)
            .current_dir(base_dir)
            .spawn()
            .map_err(|e| LauncherError::MinecraftLaunchError {
                error: e.to_string(),
            })?;

        Ok(())
    }

    /// Get available Minecraft versions
    pub async fn get_versions(&self) -> LauncherResult<Vec<MinecraftVersion>> {
        let manifest_json = minecraft_api::download_version_manifest(self.downloader.client()).await?;
        let manifest = minecraft_api::parse_version_manifest(&manifest_json)?;
        
        let versions = manifest
            .versions
            .iter()
            .map(|v| MinecraftVersion {
                id: v.id.clone(),
                r#type: v.r#type.clone(),
                url: v.url.clone(),
            })
            .collect();
            
        Ok(versions)
    }

    /// Launch vanilla Minecraft (original implementation)
    async fn launch_vanilla(
        &self,
        config: &LauncherConfig,
        version: &str,
        base_dir: &str,
        window: &tauri::Window,
    ) -> LauncherResult<()> {
        // Step 1: Download and parse version manifest
        window.emit("log", "Downloading version manifest...").ok();
        let manifest_json = minecraft_api::download_version_manifest(self.downloader.client()).await?;
        let manifest = minecraft_api::parse_version_manifest(&manifest_json)?;

        // Step 2: Find the selected version
        let selected_version = manifest
            .versions
            .iter()
            .find(|v| v.id == version)
            .ok_or_else(|| LauncherError::VersionNotFound { version: version.to_string() })?;

        // Step 3: Download version JSON
        window
            .emit("log", format!("Downloading version JSON: {}", selected_version.id))
            .ok();
        let version_json_str =
            minecraft_api::download_version_json(self.downloader.client(), &selected_version.url).await?;
        let version_json = minecraft_api::parse_version_json(&version_json_str)?;

        // Step 4: Download main JAR
        window.emit("log", "Downloading main JAR...").ok();
        let jar_path = format!(
            "{}/versions/{}/{}.jar",
            base_dir, selected_version.id, selected_version.id
        );
        self.downloader
            .download_file_with_progress(&version_json.downloads.client.url, &jar_path, Some(window))
            .await?;

        // Step 5: Download libraries
        window.emit("log", "Downloading libraries...").ok();
        self.download_libraries(&version_json.libraries, base_dir, window).await?;

        // Step 6: Download assets
        window.emit("log", "Downloading assets...").ok();
        self.download_assets(&version_json.asset_index, base_dir, window).await?;

        // Step 7: Launch Minecraft
        window.emit("log", "All downloads complete! Launching Minecraft...").ok();
        self.launch_minecraft(config, &version_json, selected_version, &jar_path, base_dir).await?;

        Ok(())
    }

    /// Ensure Fabric is installed for the given Minecraft version
    async fn ensure_fabric_installed(
        &self,
        minecraft_version: &str,
        base_dir: &str,
        window: &tauri::Window,
    ) -> LauncherResult<()> {
        // Check if any Fabric version is already installed for this Minecraft version
        let versions_dir = format!("{}/versions", base_dir);
        if let Ok(entries) = std::fs::read_dir(&versions_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("fabric-loader-") && name.ends_with(&format!("-{}", minecraft_version)) {
                    let fabric_profile_path = format!("{}/versions/{}/{}.json", base_dir, name, name);
                    if std::path::Path::new(&fabric_profile_path).exists() {
                        // Check if Fabric libraries are also downloaded
                        if let Ok(profile_content) = std::fs::read_to_string(&fabric_profile_path) {
                            if let Ok(profile) = serde_json::from_str::<serde_json::Value>(&profile_content) {
                                if let Some(libraries) = profile["libraries"].as_array() {
                                    let mut all_libs_exist = true;
                                    for lib in libraries {
                                        if let Some(name) = lib["name"].as_str() {
                                            // Convert Maven coordinates to path: group:artifact:version -> group/artifact/version/artifact-version.jar
                                            let parts: Vec<&str> = name.split(':').collect();
                                            if parts.len() == 3 {
                                                let group = parts[0].replace('.', "/");
                                                let artifact = parts[1];
                                                let version = parts[2];
                                                
                                                let path = format!("{}/{}/{}/{}-{}.jar", group, artifact, version, artifact, version);
                                                let lib_path = format!("{}/libraries/{}", base_dir, path);
                                                
                                                if !std::path::Path::new(&lib_path).exists() {
                                                    all_libs_exist = false;
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    if all_libs_exist {
                                        window.emit("log", format!("✅ Fabric already installed: {}", name)).ok();
                                        return Ok(());
                                    } else {
                                        window.emit("log", format!("🔄 Fabric profile exists but libraries missing, re-downloading...")).ok();
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        window.emit("log", "📦 Installing Fabric Loader automatically...").ok();
        
        // Download Fabric installer
        self.download_and_install_fabric(minecraft_version, base_dir, window).await?;
        
        window.emit("log", "✅ Fabric installed successfully!").ok();
        Ok(())
    }

    /// Download and install Fabric using the official installer
    async fn download_and_install_fabric(
        &self,
        minecraft_version: &str,
        base_dir: &str,
        window: &tauri::Window,
    ) -> LauncherResult<()> {
        // Get latest Fabric loader version
        let fabric_api_url = "https://meta.fabricmc.net/v2/versions/loader";
        let response = self.downloader.client()
            .get(fabric_api_url)
            .send()
            .await
            .map_err(|e| LauncherError::DownloadFailed {
                url: fabric_api_url.to_string(),
                reason: e.to_string(),
            })?;

        let fabric_versions: serde_json::Value = response.json().await.map_err(|e| LauncherError::DownloadFailed {
            url: fabric_api_url.to_string(),
            reason: e.to_string(),
        })?;

        let latest_loader = fabric_versions
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|v| v["version"].as_str())
            .unwrap_or("0.15.11");

        window.emit("log", format!("📋 Using Fabric Loader version: {}", latest_loader)).ok();

        // Download Fabric profile JSON
        let profile_url = format!(
            "https://meta.fabricmc.net/v2/versions/loader/{}/{}/profile/json",
            minecraft_version, latest_loader
        );

        window.emit("log", "⬇️ Downloading Fabric profile...").ok();
        
        let profile_response = self.downloader.client()
            .get(&profile_url)
            .send()
            .await
            .map_err(|e| LauncherError::DownloadFailed {
                url: profile_url.clone(),
                reason: e.to_string(),
            })?;

        if !profile_response.status().is_success() {
            return Err(LauncherError::DownloadFailed {
                url: profile_url,
                reason: format!("HTTP {}", profile_response.status()),
            });
        }

        let profile_json = profile_response.text().await.map_err(|e| LauncherError::DownloadFailed {
            url: profile_url.clone(),
            reason: e.to_string(),
        })?;

        // Parse and save the profile
        let fabric_version_id = format!("fabric-loader-{}-{}", latest_loader, minecraft_version);
        let versions_dir = format!("{}/versions/{}", base_dir, fabric_version_id);
        let profile_path = format!("{}/{}.json", versions_dir, fabric_version_id);

        // Create directory
        std::fs::create_dir_all(&versions_dir).map_err(|e| LauncherError::FileSystemError {
            operation: "create_fabric_dir".to_string(),
            path: versions_dir.clone(),
            error: e.to_string(),
        })?;

        // Save profile
        std::fs::write(&profile_path, &profile_json).map_err(|e| LauncherError::FileSystemError {
            operation: "write_fabric_profile".to_string(),
            path: profile_path.clone(),
            error: e.to_string(),
        })?;

        window.emit("log", "💾 Fabric profile saved successfully!").ok();

        // Download Fabric libraries
        let profile: serde_json::Value = serde_json::from_str(&profile_json).map_err(|e| LauncherError::DownloadFailed {
            url: profile_url,
            reason: format!("Failed to parse JSON: {}", e),
        })?;

        if let Some(libraries) = profile["libraries"].as_array() {
            window.emit("log", "📚 Downloading Fabric libraries...").ok();
            self.download_fabric_libraries(libraries, base_dir, window).await?;
        }

        Ok(())
    }

    /// Download Fabric libraries
    async fn download_fabric_libraries(
        &self,
        libraries: &[serde_json::Value],
        base_dir: &str,
        window: &tauri::Window,
    ) -> LauncherResult<()> {
        for (i, lib) in libraries.iter().enumerate() {
            if let Some(name) = lib["name"].as_str() {
                window.emit("log", format!("📦 Downloading library {}/{}: {}", i + 1, libraries.len(), name)).ok();
                
                // Handle Fabric library format: name + url (not downloads.artifact)
                if let Some(base_url) = lib["url"].as_str() {
                    // Convert Maven coordinates to path: group:artifact:version -> group/artifact/version/artifact-version.jar
                    let parts: Vec<&str> = name.split(':').collect();
                    if parts.len() == 3 {
                        let group = parts[0].replace('.', "/");
                        let artifact = parts[1];
                        let version = parts[2];
                        
                        let path = format!("{}/{}/{}/{}-{}.jar", group, artifact, version, artifact, version);
                        let download_url = format!("{}{}", base_url, path);
                        let lib_path = format!("{}/libraries/{}", base_dir, path);
                        
                        // Create directory for library
                        if let Some(parent) = std::path::Path::new(&lib_path).parent() {
                            std::fs::create_dir_all(parent).map_err(|e| LauncherError::FileSystemError {
                                operation: "create_lib_dir".to_string(),
                                path: parent.to_string_lossy().to_string(),
                                error: e.to_string(),
                            })?;
                        }
                        
                        // Download library
                        match self.downloader.download_file_with_progress(&download_url, &lib_path, Some(window)).await {
                            Ok(_) => {
                                window.emit("log", format!("✅ Downloaded: {}", path)).ok();
                            }
                            Err(e) => {
                                window.emit("log", format!("❌ Failed to download {}: {}", path, e)).ok();
                                return Err(e);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Launch Minecraft with Fabric
    async fn launch_with_fabric(
        &self,
        config: &LauncherConfig,
        minecraft_version: &str,
        base_dir: &str,
        window: &tauri::Window,
    ) -> LauncherResult<()> {
        // First, download vanilla Minecraft resources
        self.download_vanilla_resources(config, minecraft_version, base_dir, window).await?;
        
        // Then ensure Fabric is installed
        self.ensure_fabric_installed(minecraft_version, base_dir, window).await?;
        
        // Find the installed Fabric version dynamically
        let versions_dir = format!("{}/versions", base_dir);
        let fabric_version_id = std::fs::read_dir(&versions_dir)
            .map_err(|e| LauncherError::FileSystemError {
                operation: "read_versions_dir".to_string(),
                path: versions_dir.clone(),
                error: e.to_string(),
            })?
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("fabric-loader-") && name.ends_with(&format!("-{}", minecraft_version)) {
                    Some(name)
                } else {
                    None
                }
            })
            .next()
            .ok_or_else(|| LauncherError::DownloadFailed {
                url: "local".to_string(),
                reason: "No Fabric installation found".to_string(),
            })?;
        
        let fabric_profile_path = format!("{}/versions/{}/{}.json", base_dir, fabric_version_id, fabric_version_id);
        
        window.emit("log", format!("Loading Fabric profile: {}", fabric_version_id)).ok();
        
        // Read Fabric profile
        let profile_content = std::fs::read_to_string(&fabric_profile_path).map_err(|e| LauncherError::FileSystemError {
            operation: "read_fabric_profile".to_string(),
            path: fabric_profile_path.clone(),
            error: e.to_string(),
        })?;
        
        let fabric_profile: serde_json::Value = serde_json::from_str(&profile_content).map_err(|e| LauncherError::DownloadFailed {
            url: fabric_profile_path,
            reason: format!("Failed to parse Fabric profile: {}", e),
        })?;
        
        // Build classpath with Fabric libraries
        let mut classpath = Vec::new();
        
        // Add Fabric libraries first
        if let Some(libraries) = fabric_profile["libraries"].as_array() {
            window.emit("log", format!("🔗 Building classpath with {} Fabric libraries", libraries.len())).ok();
            for lib in libraries {
                if let Some(name) = lib["name"].as_str() {
                    // Convert Maven coordinates to path: group:artifact:version -> group/artifact/version/artifact-version.jar
                    let parts: Vec<&str> = name.split(':').collect();
                    if parts.len() == 3 {
                        let group = parts[0].replace('.', "/");
                        let artifact = parts[1];
                        let version = parts[2];
                        
                        let path = format!("{}/{}/{}/{}-{}.jar", group, artifact, version, artifact, version);
                        let lib_path = format!("{}/libraries/{}", base_dir, path);
                        
                        if std::path::Path::new(&lib_path).exists() {
                            classpath.push(lib_path.clone());
                            window.emit("log", format!("✅ Added to classpath: {}", path)).ok();
                        } else {
                            window.emit("log", format!("❌ Missing library: {}", path)).ok();
                        }
                    }
                }
            }
        }
        
        // Add vanilla Minecraft libraries
        let vanilla_version_path = format!("{}/versions/{}/{}.json", base_dir, minecraft_version, minecraft_version);
        if let Ok(vanilla_content) = std::fs::read_to_string(&vanilla_version_path) {
            if let Ok(vanilla_profile) = serde_json::from_str::<serde_json::Value>(&vanilla_content) {
                if let Some(vanilla_libraries) = vanilla_profile["libraries"].as_array() {
                    for lib in vanilla_libraries {
                        if let Some(downloads) = lib["downloads"].as_object() {
                            if let Some(artifact) = downloads["artifact"].as_object() {
                                if let Some(path) = artifact["path"].as_str() {
                                    let lib_path = format!("{}/libraries/{}", base_dir, path);
                                    classpath.push(lib_path);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Add vanilla Minecraft jar
        let minecraft_jar = format!("{}/versions/{}/{}.jar", base_dir, minecraft_version, minecraft_version);
        classpath.push(minecraft_jar);
        
        let classpath_str = classpath.join(";");
        
        // Get main class from Fabric profile
        let main_class = fabric_profile["mainClass"].as_str().unwrap_or("net.fabricmc.loader.impl.launch.knot.KnotClient");
        
        window.emit("log", format!("🎯 Main class: {}", main_class)).ok();
        window.emit("log", format!("📚 Total classpath entries: {}", classpath.len())).ok();
        
        // Prepare Java command
        let java_executable = config.get_java_executable();
        let java_args = config.get_jvm_args();
        
        let mut command = Command::new(&java_executable);
        
        // Add JVM arguments
        for arg in java_args {
            command.arg(arg);
        }

        // Add Fabric-specific JVM arguments
        if let Some(arguments) = fabric_profile["arguments"].as_object() {
            if let Some(jvm_args) = arguments["jvm"].as_array() {
                for arg in jvm_args {
                    if let Some(arg_str) = arg.as_str() {
                        command.arg(arg_str);
                    }
                }
            }
        }

        // Launch with Fabric
        window.emit("log", format!("Launching Minecraft with Fabric using main class: {}", main_class)).ok();
        
        command
            .arg("-cp")
            .arg(&classpath_str)
            .arg(main_class)
            .arg("--username")
            .arg(&config.player_name)
            .arg("--version")
            .arg(minecraft_version)
            .arg("--gameDir")
            .arg(base_dir)
            .arg("--assetsDir")
            .arg(format!("{}/assets", base_dir))
            .arg("--assetIndex")
            .arg(minecraft_version)
            .arg("--uuid")
            .arg(&config.player_uuid.clone().unwrap_or_default())
            .arg("--accessToken")
            .arg("N/A")
            .arg("--userType")
            .arg("legacy")
            .arg("--versionType")
            .arg("release");

        let _output = command.current_dir(base_dir).spawn().map_err(|e| LauncherError::MinecraftLaunchError {
            error: e.to_string(),
        })?;

        Ok(())
    }

    /// Download vanilla Minecraft resources without launching
    async fn download_vanilla_resources(
        &self,
        config: &LauncherConfig,
        version: &str,
        base_dir: &str,
        window: &tauri::Window,
    ) -> LauncherResult<()> {

        // Download and parse version manifest
        window.emit("log", "Downloading version manifest...").ok();
        let manifest_json = minecraft_api::download_version_manifest(self.downloader.client()).await?;
        let manifest = minecraft_api::parse_version_manifest(&manifest_json)?;

        // Find the selected version
        let selected_version = manifest
            .versions
            .iter()
            .find(|v| v.id == version)
            .ok_or_else(|| LauncherError::VersionNotFound { version: version.to_string() })?;

        // Download version JSON
        window
            .emit("log", format!("Downloading version JSON: {}", selected_version.id))
            .ok();
        let version_json_str =
            minecraft_api::download_version_json(self.downloader.client(), &selected_version.url).await?;
        let version_json = minecraft_api::parse_version_json(&version_json_str)?;

        // Download main JAR
        window.emit("log", "Downloading main JAR...").ok();
        let jar_path = format!(
            "{}/versions/{}/{}.jar",
            base_dir, selected_version.id, selected_version.id
        );
        
        // Create version directory
        let version_dir = format!("{}/versions/{}", base_dir, selected_version.id);
        std::fs::create_dir_all(&version_dir).map_err(|e| LauncherError::FileSystemError {
            operation: "create_version_dir".to_string(),
            path: version_dir,
            error: e.to_string(),
        })?;
        
        // Save version JSON
        let version_json_path = format!("{}/versions/{}/{}.json", base_dir, selected_version.id, selected_version.id);
        std::fs::write(&version_json_path, &version_json_str).map_err(|e| LauncherError::FileSystemError {
            operation: "save_version_json".to_string(),
            path: version_json_path,
            error: e.to_string(),
        })?;
        
        self.downloader
            .download_file_with_progress(&version_json.downloads.client.url, &jar_path, Some(window))
            .await?;

        // Download libraries
        window.emit("log", "Downloading libraries...").ok();
        self.download_libraries(&version_json.libraries, base_dir, window).await?;

        // Download assets
        window.emit("log", "Downloading assets...").ok();
        self.download_assets(&version_json.asset_index, base_dir, window).await?;

        Ok(())
    }
}
