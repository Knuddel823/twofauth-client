use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const CONFIG_DIR_NAME: &str = "twofauth-client";
const CONFIG_FILE_NAME: &str = "config.toml";

fn default_language() -> String {
    "system".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server_url: String,

    #[serde(default = "default_language")]
    pub language: String,
}

impl AppConfig {
    pub fn new(server_url: impl Into<String>) -> Self {
        Self {
            server_url: server_url.into().trim_end_matches('/').to_string(),
            language: default_language(),
        }
    }

    pub fn with_language(server_url: impl Into<String>, language: impl Into<String>) -> Self {
        Self {
            server_url: server_url.into().trim_end_matches('/').to_string(),
            language: language.into(),
        }
    }
}

pub fn config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .context("Could not determine the user configuration directory")?
        .join(CONFIG_DIR_NAME);

    Ok(config_dir.join(CONFIG_FILE_NAME))
}

pub fn save_config(config: &AppConfig) -> Result<()> {
    let path = config_path()?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Could not create config directory {}", parent.display()))?;
    }

    let toml =
        toml::to_string_pretty(config).context("Could not serialize application configuration")?;

    fs::write(&path, toml)
        .with_context(|| format!("Could not write configuration to {}", path.display()))?;

    Ok(())
}

pub fn load_config() -> Result<AppConfig> {
    let path = config_path()?;

    let contents = fs::read_to_string(&path)
        .with_context(|| format!("Could not read configuration from {}", path.display()))?;

    toml::from_str(&contents)
        .with_context(|| format!("Could not parse configuration from {}", path.display()))
}

pub fn delete_config() -> Result<()> {
    let path = config_path()?;

    if path.exists() {
        fs::remove_file(&path)
            .with_context(|| format!("Could not delete configuration {}", path.display()))?;
    }

    Ok(())
}

pub fn config_exists() -> bool {
    config_path().map(|path| path.is_file()).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_roundtrip_serialization() {
        let config = AppConfig::with_language("https://2fauth.example.com/", "de");

        let serialized = toml::to_string(&config).expect("Could not serialize test config");

        let loaded: AppConfig =
            toml::from_str(&serialized).expect("Could not deserialize test config");

        assert_eq!(loaded.server_url, "https://2fauth.example.com");
        assert_eq!(loaded.language, "de");
    }

    #[test]
    fn old_config_defaults_to_system_language() {
        let old_config = r#"
server_url = "https://2fauth.example.com"
"#;

        let loaded: AppConfig =
            toml::from_str(old_config).expect("Could not deserialize old config");

        assert_eq!(loaded.server_url, "https://2fauth.example.com");
        assert_eq!(loaded.language, "system");
    }
}
