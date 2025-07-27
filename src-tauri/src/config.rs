use crate::minecraft_api::{LauncherError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherConfig {
    pub game_directory: String,
    pub java_executable: Option<String>,

    pub min_memory: u32,
    pub max_memory: u32,

    pub jvm_args: Vec<String>,

    pub player_name: String,
    pub player_uuid: Option<String>,

    pub download_timeout: u64,
    pub max_retries: u32,
    pub concurrent_downloads: u32,

    pub theme: String,
    pub show_snapshots: bool,
    pub show_beta_versions: bool,
}

impl Default for LauncherConfig {
    fn default() -> Self {
        Self {
            game_directory: "./.minecraft_fake".to_string(),
            java_executable: None,
            min_memory: 1024,
            max_memory: 4096,
            jvm_args: vec![],
            player_name: "Player".to_string(),
            player_uuid: None,
            download_timeout: 30,
            max_retries: 3,
            concurrent_downloads: 8,
            theme: "auto".to_string(),
            show_snapshots: false,
            show_beta_versions: false,
        }
    }
}

impl LauncherConfig {
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path();

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let config: LauncherConfig = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            let config = LauncherConfig::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path();

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(&config_path, content)?;
        Ok(())
    }

    fn get_config_path() -> std::path::PathBuf {
        if let Some(config_dir) = dirs::config_dir() {
            config_dir.join("satellite-launcher").join("config.json")
        } else {
            std::path::PathBuf::from("./config.json")
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.min_memory > self.max_memory {
            return Err(LauncherError::ConfigValidation {
                field: "memory".to_string(),
                message: "Min memory cannot be greater than max memory".to_string(),
            });
        }

        if self.max_memory < 512 {
            return Err(LauncherError::ConfigValidation {
                field: "max_memory".to_string(),
                message: "Max memory must be at least 512MB".to_string(),
            });
        }

        if self.game_directory.is_empty() {
            return Err(LauncherError::ConfigValidation {
                field: "game_directory".to_string(),
                message: "Game directory cannot be empty".to_string(),
            });
        }

        if self.player_name.is_empty() {
            return Err(LauncherError::ConfigValidation {
                field: "player_name".to_string(),
                message: "Player name cannot be empty".to_string(),
            });
        }

        Ok(())
    }

    pub fn get_jvm_args(&self) -> Vec<String> {
        let mut args = vec![
            format!("-Xms{}M", self.min_memory),
            format!("-Xmx{}M", self.max_memory),
        ];

        args.extend(self.jvm_args.clone());

        args
    }

    pub fn get_java_executable(&self) -> String {
        self.java_executable
            .clone()
            .unwrap_or_else(|| "java".to_string())
    }
}
