use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub const MAX_GLOBAL_RUNNING_TURNS: usize = 3;
pub const GLOBAL_PARALLEL_FULL_MSG: &str = "当前已有 3 个任务正在执行，请稍后重试。";

#[derive(Clone, Debug)]
pub struct ActiveRunSlot {
    pub session_id: String,
    pub turn_id: String,
    pub project_id: String,
}

pub struct RunSlotGuard {
    limiter: RunLimiter,
    session_id: String,
}

impl Drop for RunSlotGuard {
    fn drop(&mut self) {
        self.limiter.release(&self.session_id);
    }
}

#[derive(Clone, Default)]
pub struct RunLimiter {
    inner: Arc<Mutex<HashMap<String, ActiveRunSlot>>>,
}

impl RunLimiter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn occupied_count(&self) -> usize {
        self.inner.lock().map(|g| g.len()).unwrap_or(0)
    }

    pub fn preflight(&self, session_id: &str) -> Result<(), String> {
        let guard = self.inner.lock().map_err(|e| e.to_string())?;
        if guard.contains_key(session_id) {
            return Err("当前会话正在执行任务，请等待完成或先停止。".into());
        }
        if guard.len() >= MAX_GLOBAL_RUNNING_TURNS {
            return Err(GLOBAL_PARALLEL_FULL_MSG.into());
        }
        Ok(())
    }

    pub fn acquire(
        &self,
        session_id: String,
        turn_id: String,
        project_id: String,
    ) -> Result<RunSlotGuard, String> {
        let mut guard = self.inner.lock().map_err(|e| e.to_string())?;
        if guard.contains_key(&session_id) {
            return Err("当前会话正在执行任务，请等待完成或先停止。".into());
        }
        if guard.len() >= MAX_GLOBAL_RUNNING_TURNS {
            return Err(GLOBAL_PARALLEL_FULL_MSG.into());
        }
        guard.insert(
            session_id.clone(),
            ActiveRunSlot {
                session_id: session_id.clone(),
                turn_id,
                project_id,
            },
        );
        Ok(RunSlotGuard {
            limiter: self.clone(),
            session_id,
        })
    }

    fn release(&self, session_id: &str) {
        if let Ok(mut guard) = self.inner.lock() {
            guard.remove(session_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fourth_slot_rejected() {
        let limiter = RunLimiter::new();
        let _g1 = limiter
            .acquire("s0".into(), "t0".into(), "p1".into())
            .unwrap();
        let _g2 = limiter
            .acquire("s1".into(), "t1".into(), "p1".into())
            .unwrap();
        let _g3 = limiter
            .acquire("s2".into(), "t2".into(), "p1".into())
            .unwrap();
        assert_eq!(
            limiter.preflight("s4").unwrap_err(),
            GLOBAL_PARALLEL_FULL_MSG
        );
    }

    #[test]
    fn guard_drop_releases_slot() {
        let limiter = RunLimiter::new();
        {
            let _g = limiter
                .acquire("s1".into(), "t1".into(), "p1".into())
                .unwrap();
            assert_eq!(limiter.occupied_count(), 1);
        }
        assert_eq!(limiter.occupied_count(), 0);
        assert!(limiter.preflight("s2").is_ok());
    }

    #[test]
    fn same_session_rejected_while_occupied() {
        let limiter = RunLimiter::new();
        let _g = limiter
            .acquire("s1".into(), "t1".into(), "p1".into())
            .unwrap();
        assert!(limiter.preflight("s1").is_err());
    }
}
