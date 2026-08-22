use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use reqwest::Url;
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

    #[serde(default)]
    pub allow_insecure_http: bool,
}

impl AppConfig {
    pub fn with_settings(
        server_url: impl Into<String>,
        language: impl Into<String>,
        allow_insecure_http: bool,
    ) -> Self {
        Self {
            server_url: normalize_server_url(server_url),
            language: language.into(),
            allow_insecure_http,
        }
    }
}

fn normalize_server_url(server_url: impl Into<String>) -> String {
    server_url.into().trim().trim_end_matches('/').to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerUrlError {
    Empty,
    InvalidUrl,
    MissingHost,
    InsecureHttpDisabled,
    UnsupportedScheme,
}

pub fn validate_server_url(
    server_url: &str,
    allow_insecure_http: bool,
) -> std::result::Result<(), ServerUrlError> {
    let server_url = server_url.trim();

    if server_url.is_empty() {
        return Err(ServerUrlError::Empty);
    }

    let url = Url::parse(server_url).map_err(|_| ServerUrlError::InvalidUrl)?;

    if url.host_str().is_none() {
        return Err(ServerUrlError::MissingHost);
    }

    match url.scheme() {
        "https" => Ok(()),
        "http" if allow_insecure_http => Ok(()),
        "http" => Err(ServerUrlError::InsecureHttpDisabled),
        _ => Err(ServerUrlError::UnsupportedScheme),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_roundtrip_serialization() {
        let config = AppConfig::with_settings("https://2fauth.example.com/", "de", false);

        let serialized = toml::to_string(&config).expect("Could not serialize test config");

        let loaded: AppConfig =
            toml::from_str(&serialized).expect("Could not deserialize test config");

        assert_eq!(loaded.server_url, "https://2fauth.example.com");
        assert_eq!(loaded.language, "de");
        assert!(!loaded.allow_insecure_http);
    }

    #[test]
    fn old_config_defaults_to_secure_settings() {
        let old_config = r#"
server_url = "https://2fauth.example.com"
"#;

        let loaded: AppConfig =
            toml::from_str(old_config).expect("Could not deserialize old config");

        assert_eq!(loaded.server_url, "https://2fauth.example.com");
        assert_eq!(loaded.language, "system");
        assert!(!loaded.allow_insecure_http);
    }

    #[test]
    fn https_is_allowed_by_default() {
        assert!(validate_server_url("https://2fauth.example.com", false).is_ok());
    }

    #[test]
    fn http_is_rejected_by_default() {
        assert!(validate_server_url("http://2fauth.example.com", false).is_err());
    }

    #[test]
    fn http_can_be_explicitly_allowed() {
        assert!(validate_server_url("http://2fauth.example.com", true).is_ok());
    }

    #[test]
    fn unsupported_url_scheme_is_rejected() {
        assert!(validate_server_url("ftp://2fauth.example.com", true).is_err());
    }

    #[test]
    fn invalid_url_is_rejected() {
        assert!(validate_server_url("not a server address", false).is_err());
    }
}
