use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

pub const TURN_CANCELLED: &str = "turn cancelled";

#[derive(Clone, Debug)]
pub struct CancelSignal(Arc<AtomicBool>);

impl CancelSignal {
    pub fn new() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }

    pub fn cancel(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

impl Default for CancelSignal {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct ActiveTurn {
    pub session_id: String,
    pub turn_id: String,
    pub project_id: String,
    pub cancel: CancelSignal,
}

#[derive(Clone, Default)]
pub struct TurnRegistry {
    inner: Arc<Mutex<TurnRegistryInner>>,
}

#[derive(Default)]
struct TurnRegistryInner {
    active: HashMap<String, ActiveTurn>,
    reserved: HashMap<String, String>,
}

impl TurnRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    fn check_turn_can_start(
        guard: &TurnRegistryInner,
        session_id: &str,
        _project_id: &str,
    ) -> Result<(), String> {
        if guard.active.contains_key(session_id) || guard.reserved.contains_key(session_id) {
            return Err("当前会话正在执行任务，请等待完成或先停止。".into());
        }
        Ok(())
    }

    pub fn preflight_turn_start(&self, session_id: &str, project_id: &str) -> Result<(), String> {
        let guard = self.inner.lock().map_err(|e| e.to_string())?;
        Self::check_turn_can_start(&guard, session_id, project_id)
    }

    pub fn register(
        &self,
        session_id: String,
        turn_id: String,
        project_id: String,
    ) -> Result<CancelSignal, String> {
        let mut guard = self.inner.lock().map_err(|e| e.to_string())?;
        Self::check_turn_can_start(&guard, &session_id, &project_id)?;
        let cancel = CancelSignal::new();
        guard.active.insert(
            session_id.clone(),
            ActiveTurn {
                session_id,
                turn_id,
                project_id,
                cancel: cancel.clone(),
            },
        );
        Ok(cancel)
    }

    pub fn reserve_resume(&self, session_id: String, project_id: String) -> Result<(), String> {
        let mut guard = self.inner.lock().map_err(|e| e.to_string())?;
        if guard.active.contains_key(&session_id) {
            return Err("当前会话正在执行任务，请等待完成或先停止。".into());
        }
        guard.reserved.insert(session_id, project_id);
        Ok(())
    }

    pub fn register_reserved(
        &self,
        session_id: String,
        turn_id: String,
        project_id: String,
    ) -> Result<CancelSignal, String> {
        let mut guard = self.inner.lock().map_err(|e| e.to_string())?;
        match guard.reserved.get(&session_id) {
            Some(reserved_project) if reserved_project == &project_id => {}
            _ => {
                drop(guard);
                return self.register(session_id, turn_id, project_id);
            }
        }
        let cancel = CancelSignal::new();
        guard.reserved.remove(&session_id);
        guard.active.insert(
            session_id.clone(),
            ActiveTurn {
                session_id,
                turn_id,
                project_id,
                cancel: cancel.clone(),
            },
        );
        Ok(cancel)
    }

    pub fn unregister(&self, session_id: &str) {
        if let Ok(mut guard) = self.inner.lock() {
            guard.active.remove(session_id);
        }
    }

    pub fn unreserve(&self, session_id: &str) {
        if let Ok(mut guard) = self.inner.lock() {
            guard.reserved.remove(session_id);
        }
    }

    pub fn cancel(&self, session_id: &str) -> Result<(), String> {
        let guard = self.inner.lock().map_err(|e| e.to_string())?;
        let active = guard
            .active
            .get(session_id)
            .ok_or_else(|| "当前没有进行中的任务。".to_string())?;
        active.cancel.cancel();
        Ok(())
    }

    pub fn active_for_session(&self, session_id: &str) -> Option<ActiveTurn> {
        self.inner.lock().ok()?.active.get(session_id).cloned()
    }

    pub fn is_session_active(&self, session_id: &str) -> bool {
        self.inner
            .lock()
            .ok()
            .is_some_and(|g| g.active.contains_key(session_id))
    }
}

pub fn is_session_busy_user_error(err: &str) -> bool {
    err.contains("当前会话正在执行任务")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preflight_matches_register_rules() {
        let registry = TurnRegistry::new();
        registry
            .register("s1".into(), "t1".into(), "p1".into())
            .unwrap();
        assert!(registry.preflight_turn_start("s1", "p1").is_err());
        assert!(registry.preflight_turn_start("s2", "p1").is_ok());
        assert!(registry.preflight_turn_start("s2", "p2").is_ok());
    }

    #[test]
    fn register_rejects_same_session_twice() {
        let registry = TurnRegistry::new();
        registry
            .register("s1".into(), "t1".into(), "p1".into())
            .unwrap();
        assert!(registry
            .register("s1".into(), "t2".into(), "p1".into())
            .is_err());
    }

    #[test]
    fn second_session_same_project_allowed() {
        let registry = TurnRegistry::new();
        registry
            .register("s1".into(), "t1".into(), "p1".into())
            .unwrap();
        assert!(registry
            .register("s2".into(), "t2".into(), "p1".into())
            .is_ok());
    }

    #[test]
    fn reserved_resume_allows_other_project_sessions() {
        let registry = TurnRegistry::new();
        registry.reserve_resume("s1".into(), "p1".into()).unwrap();
        assert!(registry.preflight_turn_start("s1", "p1").is_err());
        assert!(registry.preflight_turn_start("s2", "p1").is_ok());
        registry
            .register_reserved("s1".into(), "t1".into(), "p1".into())
            .unwrap();
        assert!(registry.is_session_active("s1"));
    }

    #[test]
    fn reserved_register_succeeds_after_reserve_resume() {
        let registry = TurnRegistry::new();
        registry.reserve_resume("s1".into(), "p1".into()).unwrap();
        assert!(registry.preflight_turn_start("s1", "p1").is_err());
        registry
            .register_reserved("s1".into(), "t1".into(), "p1".into())
            .unwrap();
        assert!(registry.is_session_active("s1"));
        assert!(!registry.preflight_turn_start("s1", "p1").is_ok());
    }

    #[test]
    fn unregister_is_idempotent() {
        let registry = TurnRegistry::new();
        registry
            .register("s1".into(), "t1".into(), "p1".into())
            .unwrap();
        registry.unregister("s1");
        registry.unregister("s1");
        assert!(!registry.is_session_active("s1"));
    }

    #[test]
    fn cancel_sets_signal() {
        let registry = TurnRegistry::new();
        let signal = registry
            .register("s1".into(), "t1".into(), "p1".into())
            .unwrap();
        assert!(!signal.is_cancelled());
        registry.cancel("s1").unwrap();
        assert!(signal.is_cancelled());
    }
}
