use crate::core::file_locks::{normalize_project_path, FileLockRegistry, TurnFileLockStore};
use crate::core::sandbox::Sandbox;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

pub struct RuntimeWriteGate {
    registry: Arc<FileLockRegistry>,
    turn_locks: TurnFileLockStore,
    sandbox: Sandbox,
    project_id: String,
    session_id: String,
    turn_id: String,
    session_title: String,
    held_paths: Mutex<HashSet<String>>,
}

impl RuntimeWriteGate {
    pub fn new(
        registry: Arc<FileLockRegistry>,
        turn_locks: TurnFileLockStore,
        sandbox: &Sandbox,
        project_id: String,
        session_id: String,
        turn_id: String,
        session_title: String,
    ) -> Self {
        Self {
            registry,
            turn_locks,
            sandbox: sandbox.clone(),
            project_id,
            session_id,
            turn_id,
            session_title,
            held_paths: Mutex::new(HashSet::new()),
        }
    }

    pub fn before_write(&self, user_path: &str) -> Result<(), String> {
        let rel = normalize_project_path(&self.sandbox, user_path).map_err(|e| e.to_string())?;
        let mut held = self.held_paths.lock().map_err(|e| e.to_string())?;
        if held.contains(&rel) {
            return Ok(());
        }
        let guard = self
            .registry
            .try_acquire_write(
                &self.project_id,
                &self.session_id,
                &self.turn_id,
                &self.session_title,
                &rel,
            )
            .map_err(|e| e.message)?;
        self.turn_locks.hold(guard)?;
        held.insert(rel);
        Ok(())
    }
}
