// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod config;
mod minecraft_api;
use crate::config::LauncherConfig;
use crate::minecraft_api::{LauncherError, MinecraftVersion};
use glob::glob;
use reqwest::Client;
use std::process::Command;
use tauri::Emitter;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            start_launcher,
            get_versions,
            get_config,
            update_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn get_versions() -> Result<Vec<MinecraftVersion>, String> {
    let client = Client::new();
    let manifest_json = minecraft_api::download_version_manifest(&client)
        .await
        .map_err(|e| e.to_string())?;
    let manifest =
        minecraft_api::parse_version_manifest(&manifest_json).map_err(|e| e.to_string())?;
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

#[tauri::command]
async fn start_launcher(window: tauri::Window, version: String) -> Result<(), String> {
    let result = start_launcher_internal(window.clone(), version).await;

    match result {
        Ok(()) => Ok(()),
        Err(e) => {
            window.emit("error", format!("Error: {}", e)).ok();
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn get_config() -> Result<LauncherConfig, String> {
    let config = LauncherConfig::load().map_err(|e| e.to_string())?;
    Ok(config)
}

#[tauri::command]
async fn update_config(config: LauncherConfig) -> Result<(), String> {
    config.save().map_err(|e| e.to_string())
}

async fn start_launcher_internal(
    window: tauri::Window,
    version: String,
) -> Result<(), LauncherError> {
    let config = LauncherConfig::load().map_err(|e| LauncherError::ConfigValidation {
        field: "config".to_string(),
        message: e.to_string(),
    })?;

    let base_dir = &config.game_directory;
    let client = Client::new();

    window.emit("log", "Downloading version manifest...").ok();
    let manifest_json = minecraft_api::download_version_manifest(&client).await?;
    let manifest = minecraft_api::parse_version_manifest(&manifest_json)?;

    let selected_version = manifest.versions.iter().find(|v| v.id == version);
    let selected_version = match selected_version {
        Some(v) => v,
        None => {
            return Err(LauncherError::VersionNotFound { version });
        }
    };

    window
        .emit(
            "log",
            format!("Downloading version JSON: {}", selected_version.id),
        )
        .ok();
    let version_json_str =
        minecraft_api::download_version_json(&client, &selected_version.url).await?;
    let version_json = minecraft_api::parse_version_json(&version_json_str)?;

    window.emit("log", "Downloading main JAR...").ok();
    let jar_path = format!(
        "{}/versions/{}/{}.jar",
        base_dir, selected_version.id, selected_version.id
    );
    let jar_url = &version_json.downloads.client.url;
    minecraft_api::download_file(&client, jar_url, &jar_path).await?;
    window
        .emit("log", format!("Jar downloaded to: {}", jar_path))
        .ok();

    use sha1::{Digest, Sha1};
    use std::fs::File;
    use std::io::Read;

    for lib in &version_json.libraries {
        if let Some(downloads) = &lib.downloads {
            if let Some(artifact) = &downloads.artifact {
                if let Some(lib_path_rel) = &artifact.path {
                    let lib_path = format!("{}/libraries/{}", base_dir, lib_path_rel);
                    let mut needs_download = true;
                    // Verify hash if file exists and has expected hash
                    if let Some(expected_sha1) = artifact.sha1.as_ref() {
                        if std::path::Path::new(&lib_path).exists() {
                            if let Ok(mut file) = File::open(&lib_path) {
                                let mut hasher = Sha1::new();
                                let mut buffer = [0u8; 8192]; //chunks of 8192 bytes
                                loop {
                                    let n = file.read(&mut buffer).unwrap_or(0);
                                    if n == 0 {
                                        break;
                                    }
                                    hasher.update(&buffer[..n]);
                                }
                                let actual_sha1 = format!("{:x}", hasher.finalize());
                                if &actual_sha1 == expected_sha1 {
                                    needs_download = false;
                                    window.emit("log", format!("Library OK: {}", lib_path)).ok();
                                } else {
                                    window
                                        .emit(
                                            "log",
                                            format!("Hash mismatch, re-downloading: {}", lib_path),
                                        )
                                        .ok();
                                }
                            }
                        }
                    } else if !std::path::Path::new(&lib_path).exists() {
                        needs_download = true;
                    } else {
                        // No hash, file exists
                        needs_download = false;
                    }
                    if needs_download {
                        minecraft_api::download_file(&client, &artifact.url, &lib_path).await?;
                        window
                            .emit("log", format!("Downloaded library: {}", lib_path))
                            .ok();
                    }
                }
            }
        }
    }

    window.emit("log", "Downloading assets in parallel...").ok();
    let assets_index_path = format!(
        "{}/assets/indexes/{}.json",
        base_dir, version_json.asset_index.id
    );
    minecraft_api::download_file(&client, &version_json.asset_index.url, &assets_index_path)
        .await?;

    let asset_futures =
        minecraft_api::download_assets(&client, &assets_index_path, base_dir, window.clone());
    asset_futures.await?;

    window
        .emit("log", "All assets downloaded! Launching Minecraft...")
        .ok();

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

    classpath.push(jar_path.clone());
    let classpath_str = classpath.join(";");

    let java_executable = config.get_java_executable();
    let java_args = config.get_jvm_args();
    let mut command = Command::new(&java_executable);
    for args in java_args {
        command.arg(args);
    }
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
        .arg(&config.player_uuid.unwrap_or_default())
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

    window
        .emit(
            "log",
            format!("Java command launched, check if Minecraft window opens."),
        )
        .unwrap();
    Ok(())
}
