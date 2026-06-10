use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

const CONFIG_FILE: &str = "config.toml";
const LEGACY_CONFIG_FILE: &str = "config.json";

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

    pub fn new_default() -> Result<Self, SecretError> {
        let dir = home_config_dir()?;
        fs::create_dir_all(&dir)?;
        restrict_dir_permissions(&dir)?;
        let secrets = Self::new(dir.join(CONFIG_FILE));
        secrets.migrate_legacy_json_config()?;
        Ok(secrets)
    }

    pub fn config_dir() -> Result<PathBuf, SecretError> {
        home_config_dir()
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

    /// Remove API keys stored in the legacy OS keychain (pre file-based config).
    pub fn cleanup_legacy_keychain() {
        #[cfg(target_os = "macos")]
        cleanup_macos_keychain();
    }

    fn migrate_legacy_json_config(&self) -> Result<(), SecretError> {
        let Some(dir) = self.config_path.parent() else {
            return Ok(());
        };
        let legacy_path = dir.join(LEGACY_CONFIG_FILE);
        if !legacy_path.exists() || self.config_path.exists() {
            return Ok(());
        }

        let raw = fs::read_to_string(&legacy_path)?;
        let config: SecretsConfig = serde_json::from_str(&raw)
            .map_err(|e| SecretError::Config(format!("legacy config.json invalid: {e}")))?;
        self.save_config(&config)?;
        fs::remove_file(&legacy_path)?;
        Ok(())
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

fn home_config_dir() -> Result<PathBuf, SecretError> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|e| SecretError::Config(format!("home directory not found: {e}")))?;
    Ok(PathBuf::from(home).join(".doc-agent"))
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

#[cfg(target_os = "macos")]
fn cleanup_macos_keychain() {
    for account in ["deepseek", "kimi", "mock", "doc-agent"] {
        let _ = std::process::Command::new("security")
            .args(["delete-generic-password", "-s", "doc-agent", "-a", account])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }
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
    fn migrates_legacy_json_config() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join(LEGACY_CONFIG_FILE),
            r#"{"api_keys":{"kimi":"sk-kimi"}}"#,
        )
        .unwrap();

        let secrets = Secrets::new(dir.path().join(CONFIG_FILE));
        secrets.migrate_legacy_json_config().unwrap();

        assert_eq!(
            secrets.get_api_key("kimi").unwrap().as_deref(),
            Some("sk-kimi")
        );
        assert!(!dir.path().join(LEGACY_CONFIG_FILE).exists());
        assert!(dir.path().join(CONFIG_FILE).exists());
    }
}
