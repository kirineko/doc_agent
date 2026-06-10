use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

const CONFIG_FILE: &str = "config.toml";

#[derive(Debug, Error)]
pub enum SecretError {
    #[error("config error: {0}")]
    Config(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("toml error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("toml serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct SecretsConfig {
    #[serde(default)]
    api_keys: HashMap<String, String>,
}

#[derive(Clone)]
pub struct Secrets {
    config_path: PathBuf,
}

impl Secrets {
    pub fn new(config_path: PathBuf) -> Self {
        Self { config_path }
    }

    pub fn open_in_data_dir(data_dir: PathBuf) -> Result<Self, SecretError> {
        fs::create_dir_all(&data_dir)?;
        restrict_dir_permissions(&data_dir)?;
        Ok(Self::new(data_dir.join(CONFIG_FILE)))
    }

    pub fn set_api_key(&self, provider: &str, value: &str) -> Result<(), SecretError> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(SecretError::Config("api key is empty".into()));
        }
        let mut config = self.load_config()?;
        config
            .api_keys
            .insert(provider.to_string(), trimmed.to_string());
        self.save_config(&config)
    }

    pub fn get_api_key(&self, provider: &str) -> Result<Option<String>, SecretError> {
        let config = self.load_config()?;
        Ok(config.api_keys.get(provider).cloned())
    }

    pub fn has_api_key(&self, provider: &str) -> Result<bool, SecretError> {
        Ok(self.get_api_key(provider)?.is_some())
    }

    pub fn clear_api_key(&self, provider: &str) -> Result<(), SecretError> {
        let mut config = self.load_config()?;
        config.api_keys.remove(provider);
        self.save_config(&config)
    }

    fn load_config(&self) -> Result<SecretsConfig, SecretError> {
        if !self.config_path.exists() {
            return Ok(SecretsConfig::default());
        }
        let raw = fs::read_to_string(&self.config_path)?;
        Ok(toml::from_str(&raw)?)
    }

    fn save_config(&self, config: &SecretsConfig) -> Result<(), SecretError> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
            restrict_dir_permissions(parent)?;
        }
        let raw = toml::to_string_pretty(config)?;
        fs::write(&self.config_path, raw)?;
        restrict_file_permissions(&self.config_path)?;
        Ok(())
    }
}

#[cfg(unix)]
fn restrict_dir_permissions(path: &Path) -> Result<(), SecretError> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    Ok(())
}

#[cfg(not(unix))]
fn restrict_dir_permissions(_path: &Path) -> Result<(), SecretError> {
    Ok(())
}

#[cfg(unix)]
fn restrict_file_permissions(path: &Path) -> Result<(), SecretError> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    Ok(())
}

#[cfg(not(unix))]
fn restrict_file_permissions(_path: &Path) -> Result<(), SecretError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn api_key_roundtrip_via_config_file() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join(CONFIG_FILE);
        let secrets = Secrets::new(config_path);
        let provider = "deepseek";

        assert!(!secrets.has_api_key(provider).unwrap());
        secrets.set_api_key(provider, "sk-test-key").unwrap();
        assert!(secrets.has_api_key(provider).unwrap());
        assert_eq!(
            secrets.get_api_key(provider).unwrap().as_deref(),
            Some("sk-test-key")
        );

        let saved = fs::read_to_string(dir.path().join(CONFIG_FILE)).unwrap();
        assert!(saved.contains("[api_keys]"));
        assert!(saved.contains("sk-test-key"));

        secrets.clear_api_key(provider).unwrap();
        assert!(!secrets.has_api_key(provider).unwrap());
    }

    #[test]
    fn open_in_data_dir_uses_app_data_path() {
        let dir = tempdir().unwrap();
        let secrets = Secrets::open_in_data_dir(dir.path().to_path_buf()).unwrap();
        secrets.set_api_key("kimi", "sk-kimi").unwrap();
        assert!(dir.path().join(CONFIG_FILE).exists());
        assert_eq!(
            secrets.get_api_key("kimi").unwrap().as_deref(),
            Some("sk-kimi")
        );
    }
}
