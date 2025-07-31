use std::path::PathBuf;
use crate::core::error::GitNavigatorError;

pub fn get_config_directory() -> Result<PathBuf, GitNavigatorError> {
    let base = match std::env::consts::OS {
        "linux" | "freebsd" | "netbsd" | "openbsd" => {
            std::env::var("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| dirs::home_dir().unwrap_or_default().join(".config"))
        },
        "macos" => {
            dirs::home_dir()
                .unwrap_or_default()
                .join("Library/Application Support")
        },
        "windows" => {
            dirs::config_dir().unwrap_or_default()
        },
        _ => dirs::config_dir().unwrap_or_default(),
    };
    
    Ok(base.join("git-navigator"))
}

pub fn get_cache_directory() -> Result<PathBuf, GitNavigatorError> {
    let base = match std::env::consts::OS {
        "linux" | "freebsd" | "netbsd" | "openbsd" => {
            std::env::var("XDG_CACHE_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| dirs::home_dir().unwrap_or_default().join(".cache"))
        },
        "macos" => {
            dirs::home_dir()
                .unwrap_or_default()
                .join("Library/Caches")
        },
        "windows" => {
            dirs::cache_dir().unwrap_or_default()
        },
        _ => dirs::cache_dir().unwrap_or_default(),
    };
    
    Ok(base.join("git-navigator"))
}