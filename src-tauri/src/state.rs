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
    pub fn new(data_dir: PathBuf) -> Result<Self, String> {
        std::fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
        let store = Store::open(data_dir.join("doc_agent.db")).map_err(|e| e.to_string())?;
        Ok(Self {
            store: Arc::new(Mutex::new(store)),
            secrets: Secrets::open_in_data_dir(data_dir).map_err(|e| e.to_string())?,
            tools: ToolRegistry::default_tools(),
        })
    }
}
