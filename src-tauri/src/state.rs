use crate::core::secrets::Secrets;
use crate::core::store::Store;
use crate::tools::registry::ToolRegistry;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AppState {
    pub store: Arc<Mutex<Store>>,
    pub secrets: Secrets,
    pub tools: ToolRegistry,
}

impl AppState {
    pub fn new(db_path: PathBuf) -> Result<Self, String> {
        let store = Store::open(db_path).map_err(|e| e.to_string())?;
        Ok(Self {
            store: Arc::new(Mutex::new(store)),
            secrets: Secrets::new_default().map_err(|e| e.to_string())?,
            tools: ToolRegistry::default_tools(),
        })
    }
}
