use crate::core::{secrets::Secrets, store::Store};

const SETTING_KEY: &str = "web_search_enabled";

pub fn is_web_search_active(secrets: &Secrets, store: &Store) -> Result<bool, String> {
    if !secrets.has_api_key("tavily").map_err(|e| e.to_string())? {
        return Ok(false);
    }
    match store.get_setting(SETTING_KEY).map_err(|e| e.to_string())? {
        Some(value) => Ok(value == "true"),
        None => Ok(true),
    }
}

pub fn set_web_search_enabled(store: &Store, enabled: bool) -> Result<(), String> {
    store
        .set_setting(SETTING_KEY, if enabled { "true" } else { "false" })
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::store::Store;
    use tempfile::tempdir;

    fn test_secrets(dir: &std::path::Path) -> Secrets {
        Secrets::new(dir.join("config.toml"))
    }

    #[test]
    fn inactive_without_key() {
        let dir = tempdir().unwrap();
        let store = Store::open(dir.path().join("app.db")).unwrap();
        let secrets = test_secrets(dir.path());
        assert!(!is_web_search_active(&secrets, &store).unwrap());
    }

    #[test]
    fn active_by_default_when_key_present() {
        let dir = tempdir().unwrap();
        let store = Store::open(dir.path().join("app.db")).unwrap();
        let secrets = test_secrets(dir.path());
        secrets.set_api_key("tavily", "tvly-test").unwrap();
        assert!(is_web_search_active(&secrets, &store).unwrap());
    }

    #[test]
    fn preference_can_disable_while_key_remains() {
        let dir = tempdir().unwrap();
        let store = Store::open(dir.path().join("app.db")).unwrap();
        let secrets = test_secrets(dir.path());
        secrets.set_api_key("tavily", "tvly-test").unwrap();
        set_web_search_enabled(&store, false).unwrap();
        assert!(!is_web_search_active(&secrets, &store).unwrap());
        assert!(secrets.has_api_key("tavily").unwrap());
    }

    #[test]
    fn preference_can_be_turned_back_on() {
        let dir = tempdir().unwrap();
        let store = Store::open(dir.path().join("app.db")).unwrap();
        let secrets = test_secrets(dir.path());
        secrets.set_api_key("tavily", "tvly-test").unwrap();
        set_web_search_enabled(&store, false).unwrap();
        assert!(!is_web_search_active(&secrets, &store).unwrap());
        set_web_search_enabled(&store, true).unwrap();
        assert!(is_web_search_active(&secrets, &store).unwrap());
    }
}
