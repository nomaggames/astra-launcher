use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LauncherConfig {
    pub fullscreen: bool,
}

impl Default for LauncherConfig {
    fn default() -> Self {
        Self {
            fullscreen: true, // Default to fullscreen
        }
    }
}

fn get_config_path() -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let config_dir = dirs::config_dir()
        .ok_or("Could not find config directory")?;
    let launcher_config_dir = config_dir.join("astra-launcher");
    fs::create_dir_all(&launcher_config_dir)?;
    Ok(launcher_config_dir.join("config.json"))
}

pub fn load_config() -> Result<LauncherConfig, Box<dyn std::error::Error + Send + Sync>> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        // Create default config if it doesn't exist
        let default_config = LauncherConfig::default();
        save_config(&default_config)?;
        return Ok(default_config);
    }

    let content = fs::read_to_string(config_path)?;
    let config: LauncherConfig = serde_json::from_str(&content)?;
    Ok(config)
}

pub fn save_config(config: &LauncherConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config_path = get_config_path()?;
    let content = serde_json::to_string_pretty(config)?;
    fs::write(config_path, content)?;
    Ok(())
}
