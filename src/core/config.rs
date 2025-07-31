use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::core::error::GitNavigatorError;
use crate::core::dirs::get_config_directory;

#[derive(Serialize, Deserialize, Debug)]
pub struct RepositoryConfig {
    pub owner: String,
    pub name: String,
    pub bin_name: String,
}

impl Default for RepositoryConfig {
    fn default() -> Self {
        Self {
            owner: "blitux".to_string(),
            name: "git-navigator".to_string(),
            bin_name: "git-navigator".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateConfig {
    pub last_check: Option<chrono::DateTime<chrono::Utc>>,
    pub auto_check_enabled: bool,
    pub skip_version: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InstallConfig {
    pub installed_version: String,
    pub install_date: chrono::DateTime<chrono::Utc>,
    pub binary_path: PathBuf,
    pub repository: RepositoryConfig,
    pub update_config: UpdateConfig,
}

impl InstallConfig {
    pub fn load_or_create() -> Result<Self, GitNavigatorError> {
        let config_dir = get_config_directory()?;
        let config_file = config_dir.join("config.json");
        
        if config_file.exists() {
            let content = std::fs::read_to_string(&config_file)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            let config = Self {
                installed_version: env!("CARGO_PKG_VERSION").to_string(),
                install_date: chrono::Utc::now(),
                binary_path: std::env::current_exe().unwrap_or_default(),
                repository: RepositoryConfig::default(),
                update_config: UpdateConfig {
                    last_check: None,
                    auto_check_enabled: false,
                    skip_version: None,
                },
            };
            config.save()?;
            Ok(config)
        }
    }
    
    pub fn save(&self) -> Result<(), GitNavigatorError> {
        let config_dir = get_config_directory()?;
        std::fs::create_dir_all(&config_dir)?;
        
        let config_file = config_dir.join("config.json");
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&config_file, content)?;
        
        Ok(())
    }
    
    pub fn update_version(&mut self, new_version: &str) -> Result<(), GitNavigatorError> {
        self.installed_version = new_version.to_string();
        self.update_config.last_check = Some(chrono::Utc::now());
        self.save()
    }
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            last_check: None,
            auto_check_enabled: false,
            skip_version: None,
        }
    }
}