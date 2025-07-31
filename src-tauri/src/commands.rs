use crate::config::LauncherConfig;
use crate::launcher::MinecraftLauncher;
use crate::minecraft_api::MinecraftVersion;
use crate::mods::{ModManager, ModInfo, ModSearchResult, ModLoader, search_mods};
use tauri::Emitter;

/// Get available Minecraft versions
#[tauri::command]
pub async fn get_versions() -> Result<Vec<MinecraftVersion>, String> {
    let launcher = MinecraftLauncher::new();
    launcher.get_versions().await.map_err(|e| e.to_string())
}

/// Start the Minecraft launcher
#[tauri::command]
pub async fn start_launcher(window: tauri::Window, version: String) -> Result<(), String> {
    let launcher = MinecraftLauncher::new();
    
    match launcher.launch_version(window.clone(), version).await {
        Ok(()) => Ok(()),
        Err(e) => {
            let error_msg = format!("Launch failed: {}", e);
            window.emit("error", &error_msg).ok();
            Err(error_msg)
        }
    }
}

/// Get launcher configuration
#[tauri::command]
pub async fn get_config() -> Result<LauncherConfig, String> {
    LauncherConfig::load().map_err(|e| e.to_string())
}

/// Update launcher configuration
#[tauri::command]
pub async fn update_config(config: LauncherConfig) -> Result<(), String> {
    config.save().map_err(|e| e.to_string())
}

/// Validate configuration before saving
#[tauri::command]
pub async fn validate_config(config: LauncherConfig) -> Result<bool, String> {
    // Basic validation logic
    if config.player_name.trim().is_empty() {
        return Err("Player name cannot be empty".to_string());
    }
    
    if config.player_name.len() > 16 {
        return Err("Player name cannot be longer than 16 characters".to_string());
    }
    
    if !config.game_directory.trim().is_empty() {
        if !std::path::Path::new(&config.game_directory).exists() {
            return Err("Game directory does not exist".to_string());
        }
    }
    
    Ok(true)
}

/// Get system information for diagnostics
#[tauri::command]
pub async fn get_system_info() -> Result<serde_json::Value, String> {
    let info = serde_json::json!({
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "java_home": std::env::var("JAVA_HOME").unwrap_or_default(),
        "user_home": dirs::home_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_default(),
    });
    
    Ok(info)
}

// MOD MANAGEMENT COMMANDS

/// Get all installed mods
#[tauri::command]
pub async fn get_installed_mods(game_directory: String) -> Result<Vec<ModInfo>, String> {
    let mod_manager = ModManager::new(&game_directory);
    mod_manager.get_installed_mods().map_err(|e| e.to_string())
}

/// Toggle mod enabled/disabled state
#[tauri::command]
pub async fn toggle_mod(game_directory: String, mod_id: String, enabled: bool) -> Result<(), String> {
    let mod_manager = ModManager::new(&game_directory);
    mod_manager.toggle_mod(&mod_id, enabled).map_err(|e| e.to_string())
}

/// Delete a mod
#[tauri::command]
pub async fn delete_mod(game_directory: String, mod_id: String) -> Result<(), String> {
    let mod_manager = ModManager::new(&game_directory);
    mod_manager.delete_mod(&mod_id).map_err(|e| e.to_string())
}

/// Install mod from file
#[tauri::command]
pub async fn install_mod_from_file(game_directory: String, file_path: String) -> Result<ModInfo, String> {
    let mod_manager = ModManager::new(&game_directory);
    mod_manager.install_mod_from_file(&file_path).map_err(|e| e.to_string())
}

/// Search for mods online
#[tauri::command]
pub async fn search_mods_online(
    query: String,
    minecraft_version: String,
    mod_loader: ModLoader,
    limit: u32,
) -> Result<Vec<ModSearchResult>, String> {
    search_mods(&query, &minecraft_version, &mod_loader, limit)
        .await
        .map_err(|e| e.to_string())
}
