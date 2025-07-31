use thiserror::Error;

#[derive(Debug, Error)]
pub enum LauncherError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Configuration validation error in field '{field}': {message}")]
    ConfigValidation { field: String, message: String },
    
    #[error("Version '{version}' not found")]
    VersionNotFound { version: String },
    
    #[error("Failed to launch Minecraft: {error}")]
    MinecraftLaunchError { error: String },
    
    #[error("File hash mismatch for {file}: expected {expected}, got {actual}")]
    HashMismatch {
        file: String,
        expected: String,
        actual: String,
    },
    
    #[error("Download failed for {url}: {reason}")]
    DownloadFailed { url: String, reason: String },
    
    #[error("Asset processing error: {message}")]
    AssetError { message: String },
    
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

impl From<LauncherError> for String {
    fn from(error: LauncherError) -> Self {
        error.to_string()
    }
}

// Conversion from minecraft_api::LauncherError to our LauncherError
impl From<crate::minecraft_api::LauncherError> for LauncherError {
    fn from(error: crate::minecraft_api::LauncherError) -> Self {
        match error {
            crate::minecraft_api::LauncherError::Network(e) => LauncherError::Network(e),
            crate::minecraft_api::LauncherError::FileSystem(e) => LauncherError::Io(e),
            crate::minecraft_api::LauncherError::JsonParsing(e) => LauncherError::Json(e),
            crate::minecraft_api::LauncherError::VersionNotFound { version } => {
                LauncherError::VersionNotFound { version }
            }
            crate::minecraft_api::LauncherError::MinecraftLaunchError { error } => {
                LauncherError::MinecraftLaunchError { error }
            }
            crate::minecraft_api::LauncherError::ConfigValidation { field, message } => {
                LauncherError::ConfigValidation { field, message }
            }
        }
    }
}

// Result type alias for convenience
pub type LauncherResult<T> = Result<T, LauncherError>;
