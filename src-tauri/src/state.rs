use crate::agent::run_limiter::RunLimiter;
use crate::agent::turn_control::TurnRegistry;
use crate::core::file_locks::FileLockRegistry;
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
    pub turns: Arc<TurnRegistry>,
    pub file_locks: Arc<FileLockRegistry>,
    pub run_limiter: Arc<RunLimiter>,
}

impl AppState {
    pub fn new(data_dir: PathBuf) -> Result<Self, String> {
        std::fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
        let store = Store::open(data_dir.join("doc_agent.db")).map_err(|e| e.to_string())?;
        Ok(Self {
            store: Arc::new(Mutex::new(store)),
            secrets: Secrets::open_in_data_dir(data_dir).map_err(|e| e.to_string())?,
            tools: ToolRegistry::default_tools(),
            turns: Arc::new(TurnRegistry::new()),
            file_locks: Arc::new(FileLockRegistry::new()),
            run_limiter: Arc::new(RunLimiter::new()),
        })
    }
}
