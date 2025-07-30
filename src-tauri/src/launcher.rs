use crate::config::LauncherConfig;
use crate::downloader::Downloader;
use crate::errors::{LauncherError, LauncherResult};
use crate::minecraft_api::{self, MinecraftVersion};
use std::process::Command;
use tauri::Emitter;

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

        // Step 1: Download and parse version manifest
        window.emit("log", "Downloading version manifest...").ok();
        let manifest_json = minecraft_api::download_version_manifest(self.downloader.client()).await?;
        let manifest = minecraft_api::parse_version_manifest(&manifest_json)?;

        // Step 2: Find the selected version
        let selected_version = manifest
            .versions
            .iter()
            .find(|v| v.id == version)
            .ok_or_else(|| LauncherError::VersionNotFound { version: version.clone() })?;

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
            .download_file_with_progress(&version_json.downloads.client.url, &jar_path, Some(&window))
            .await?;

        // Step 5: Download libraries
        window.emit("log", "Downloading libraries...").ok();
        self.download_libraries(&version_json.libraries, base_dir, &window).await?;

        // Step 6: Download assets
        window.emit("log", "Downloading assets...").ok();
        self.download_assets(&version_json.asset_index, base_dir, &window).await?;

        // Step 7: Launch Minecraft
        window.emit("log", "All downloads complete! Launching Minecraft...").ok();
        self.launch_minecraft(&config, &version_json, &selected_version, &jar_path, base_dir).await?;

        window
            .emit("log", "Minecraft launched successfully!")
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
}
