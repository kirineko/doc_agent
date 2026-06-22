use crate::core::sandbox::{Sandbox, SandboxError};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LockMode {
    Read,
    Write,
    SubtreeWrite,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FileResource {
    pub project_id: String,
    pub path: String,
}

#[derive(Clone, Debug)]
pub struct LockRequest {
    pub resource: FileResource,
    pub mode: LockMode,
}

#[derive(Debug, Clone)]
pub struct FileBusyError {
    pub path: String,
    pub message: String,
    pub blocking_session_id: String,
}

impl FileBusyError {
    pub fn to_tool_json(&self) -> Value {
        json!({
            "error": "file_busy",
            "message": self.message,
            "path": self.path,
            "blocking_session_id": self.blocking_session_id,
        })
    }
}

#[derive(Clone, Debug)]
struct HeldLock {
    request: LockRequest,
    session_id: String,
    turn_id: String,
    session_title: String,
}

#[derive(Clone, Default)]
pub struct FileLockRegistry {
    inner: Arc<Mutex<Vec<HeldLock>>>,
}

pub struct FileLockGuard {
    registry: FileLockRegistry,
    held: Vec<HeldLock>,
}

#[derive(Clone, Default)]
pub struct TurnFileLockStore {
    guards: Arc<Mutex<Vec<FileLockGuard>>>,
}

impl TurnFileLockStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn hold(&self, guard: FileLockGuard) -> Result<(), String> {
        self.guards.lock().map_err(|e| e.to_string())?.push(guard);
        Ok(())
    }

    #[cfg(test)]
    pub fn guard_count(&self) -> usize {
        self.guards.lock().map(|guards| guards.len()).unwrap_or(0)
    }
}

impl std::fmt::Debug for FileLockGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileLockGuard")
            .field("held", &self.held.len())
            .finish()
    }
}

impl FileLockRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn acquire_many(
        &self,
        project_id: &str,
        session_id: &str,
        turn_id: &str,
        session_title: &str,
        requests: Vec<LockRequest>,
    ) -> Result<FileLockGuard, FileBusyError> {
        let deduped = dedupe_requests(requests);
        let mut guard = self.inner.lock().map_err(|_| FileBusyError {
            path: String::new(),
            message: "file lock registry poisoned".into(),
            blocking_session_id: String::new(),
        })?;
        for req in &deduped {
            if let Some(blocker) = guard
                .iter()
                .find(|held| !same_turn(held, session_id, turn_id) && conflicts(held, req))
            {
                return Err(format_busy(blocker, &req.resource.path));
            }
        }
        let mut held = Vec::with_capacity(deduped.len());
        for req in deduped {
            let entry = HeldLock {
                request: LockRequest {
                    resource: FileResource {
                        project_id: project_id.to_string(),
                        ..req.resource.clone()
                    },
                    mode: req.mode,
                },
                session_id: session_id.to_string(),
                turn_id: turn_id.to_string(),
                session_title: session_title.to_string(),
            };
            if guard.iter().any(|held| {
                same_turn(held, session_id, turn_id)
                    && held.request.resource == entry.request.resource
                    && held.request.mode == entry.request.mode
            }) {
                continue;
            }
            guard.push(entry.clone());
            held.push(entry);
        }
        Ok(FileLockGuard {
            registry: self.clone(),
            held,
        })
    }

    pub fn try_acquire_write(
        &self,
        project_id: &str,
        session_id: &str,
        turn_id: &str,
        session_title: &str,
        path: &str,
    ) -> Result<FileLockGuard, FileBusyError> {
        self.acquire_many(
            project_id,
            session_id,
            turn_id,
            session_title,
            vec![LockRequest {
                resource: FileResource {
                    project_id: project_id.to_string(),
                    path: path.to_string(),
                },
                mode: LockMode::Write,
            }],
        )
    }
}

impl Drop for FileLockGuard {
    fn drop(&mut self) {
        if let Ok(mut guard) = self.registry.inner.lock() {
            for held in &self.held {
                if let Some(pos) = guard.iter().position(|h| {
                    h.session_id == held.session_id
                        && h.turn_id == held.turn_id
                        && h.request.resource == held.request.resource
                        && h.request.mode == held.request.mode
                }) {
                    guard.remove(pos);
                }
            }
        }
    }
}

pub fn normalize_project_path(sandbox: &Sandbox, user_path: &str) -> Result<String, SandboxError> {
    let trimmed = user_path.trim().replace('\\', "/");
    if trimmed.is_empty() {
        return Err(SandboxError::InvalidPath);
    }
    let path = std::path::Path::new(&trimmed);
    for comp in path.components() {
        if matches!(comp, std::path::Component::ParentDir) {
            return Err(SandboxError::InvalidPath);
        }
    }
    let candidate = if sandbox.root().join(path).exists() {
        sandbox.resolve(&trimmed)?
    } else {
        sandbox.resolve_for_write(&trimmed)?
    };
    let rel = candidate
        .strip_prefix(sandbox.root())
        .map_err(|_| SandboxError::EscapesSandbox)?;
    let normalized = rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/");
    if normalized.is_empty() {
        if trimmed == "." {
            return Ok(".".into());
        }
        return Err(SandboxError::InvalidPath);
    }
    Ok(normalized)
}

fn dedupe_requests(requests: Vec<LockRequest>) -> Vec<LockRequest> {
    let mut out = Vec::new();
    for req in requests {
        if out.iter().any(|existing: &LockRequest| {
            existing.resource == req.resource && existing.mode == req.mode
        }) {
            continue;
        }
        out.push(req);
    }
    out
}

fn is_ancestor_or_same(parent: &str, child: &str) -> bool {
    if parent == "." {
        return true;
    }
    if child == "." {
        return parent == ".";
    }
    parent == child
        || child
            .strip_prefix(parent)
            .is_some_and(|rest| rest.starts_with('/'))
}

fn conflicts(held: &HeldLock, req: &LockRequest) -> bool {
    if held.request.resource.project_id != req.resource.project_id {
        return false;
    }
    let ap = held.request.resource.path.as_str();
    let bp = req.resource.path.as_str();
    let same = ap == bp;
    let ancestor = is_ancestor_or_same(ap, bp) || is_ancestor_or_same(bp, ap);
    match (held.request.mode, req.mode) {
        (LockMode::Read, LockMode::Read) => false,
        (LockMode::Read, LockMode::Write) | (LockMode::Write, LockMode::Read) => same,
        (LockMode::Write, LockMode::Write) => same,
        (LockMode::SubtreeWrite, _) | (_, LockMode::SubtreeWrite) => ancestor,
    }
}

fn same_turn(held: &HeldLock, session_id: &str, turn_id: &str) -> bool {
    held.session_id == session_id && held.turn_id == turn_id
}

fn format_busy(blocker: &HeldLock, path: &str) -> FileBusyError {
    let title = if blocker.session_title.trim().is_empty() {
        format!(
            "{}…",
            &blocker.session_id.chars().take(8).collect::<String>()
        )
    } else {
        blocker.session_title.clone()
    };
    FileBusyError {
        path: path.to_string(),
        message: format!("当前 {path} 已被会话「{title}」占用，请稍后重试。"),
        blocking_session_id: blocker.session_id.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::runtime::write_gate::RuntimeWriteGate;
    use std::fs;
    use std::sync::Arc;
    use tempfile::tempdir;

    fn sandbox() -> Sandbox {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), "a").unwrap();
        fs::create_dir_all(dir.path().join("docs")).unwrap();
        Sandbox::new(dir.path()).unwrap()
    }

    fn req(project: &str, path: &str, mode: LockMode) -> LockRequest {
        LockRequest {
            resource: FileResource {
                project_id: project.into(),
                path: path.into(),
            },
            mode,
        }
    }

    #[test]
    fn read_read_same_file_allowed() {
        let reg = FileLockRegistry::new();
        let _a = reg
            .acquire_many(
                "p1",
                "s1",
                "t1",
                "A",
                vec![req("p1", "a.txt", LockMode::Read)],
            )
            .unwrap();
        assert!(reg
            .acquire_many(
                "p1",
                "s2",
                "t2",
                "B",
                vec![req("p1", "a.txt", LockMode::Read)]
            )
            .is_ok());
    }

    #[test]
    fn read_write_same_file_conflicts() {
        let reg = FileLockRegistry::new();
        let _a = reg
            .acquire_many(
                "p1",
                "s1",
                "t1",
                "A",
                vec![req("p1", "a.txt", LockMode::Read)],
            )
            .unwrap();
        let err = reg
            .acquire_many(
                "p1",
                "s2",
                "t2",
                "B",
                vec![req("p1", "a.txt", LockMode::Write)],
            )
            .unwrap_err();
        assert!(err.message.contains("a.txt"));
        assert!(err.message.contains("A"));
    }

    #[test]
    fn write_write_same_file_conflicts() {
        let reg = FileLockRegistry::new();
        let _a = reg
            .acquire_many(
                "p1",
                "s1",
                "t1",
                "A",
                vec![req("p1", "out.docx", LockMode::Write)],
            )
            .unwrap();
        assert!(reg
            .acquire_many(
                "p1",
                "s2",
                "t2",
                "B",
                vec![req("p1", "out.docx", LockMode::Write)]
            )
            .is_err());
    }

    #[test]
    fn subtree_conflicts_with_descendant() {
        let reg = FileLockRegistry::new();
        let _a = reg
            .acquire_many(
                "p1",
                "s1",
                "t1",
                "A",
                vec![req("p1", "unpacked", LockMode::SubtreeWrite)],
            )
            .unwrap();
        assert!(reg
            .acquire_many(
                "p1",
                "s2",
                "t2",
                "B",
                vec![req("p1", "unpacked/word/document.xml", LockMode::Write)],
            )
            .is_err());
    }

    #[test]
    fn subtree_write_on_project_root_blocks_child_paths() {
        let reg = FileLockRegistry::new();
        let _a = reg
            .acquire_many(
                "p1",
                "s1",
                "t1",
                "A",
                vec![req("p1", ".", LockMode::SubtreeWrite)],
            )
            .unwrap();
        assert!(reg
            .acquire_many(
                "p1",
                "s2",
                "t2",
                "B",
                vec![req("p1", "a.txt", LockMode::Write)],
            )
            .is_err());
        assert!(reg
            .acquire_many(
                "p1",
                "s2",
                "t2",
                "B",
                vec![req("p1", "docs/report.docx", LockMode::Write)],
            )
            .is_err());
    }

    #[test]
    fn cross_project_same_path_allowed() {
        let reg = FileLockRegistry::new();
        let _a = reg
            .acquire_many(
                "p1",
                "s1",
                "t1",
                "A",
                vec![req("p1", "a.txt", LockMode::Write)],
            )
            .unwrap();
        assert!(reg
            .acquire_many(
                "p2",
                "s2",
                "t2",
                "B",
                vec![req("p2", "a.txt", LockMode::Write)]
            )
            .is_ok());
    }

    #[test]
    fn acquire_many_all_or_none() {
        let reg = FileLockRegistry::new();
        let _a = reg
            .acquire_many(
                "p1",
                "s1",
                "t1",
                "A",
                vec![req("p1", "a.txt", LockMode::Write)],
            )
            .unwrap();
        let err = reg
            .acquire_many(
                "p1",
                "s2",
                "t2",
                "B",
                vec![
                    req("p1", "b.txt", LockMode::Write),
                    req("p1", "a.txt", LockMode::Write),
                ],
            )
            .unwrap_err();
        assert!(err.message.contains("a.txt"));
        assert!(reg
            .acquire_many(
                "p1",
                "s2",
                "t2",
                "B",
                vec![req("p1", "b.txt", LockMode::Write)]
            )
            .is_ok());
    }

    #[test]
    fn guard_drop_releases_lock() {
        let reg = FileLockRegistry::new();
        {
            let _g = reg
                .acquire_many(
                    "p1",
                    "s1",
                    "t1",
                    "A",
                    vec![req("p1", "a.txt", LockMode::Write)],
                )
                .unwrap();
        }
        assert!(reg
            .acquire_many(
                "p1",
                "s2",
                "t2",
                "B",
                vec![req("p1", "a.txt", LockMode::Write)]
            )
            .is_ok());
    }

    #[test]
    fn same_turn_can_reenter_existing_write_lock() {
        let reg = FileLockRegistry::new();
        let first = reg
            .acquire_many(
                "p1",
                "s1",
                "t1",
                "A",
                vec![req("p1", "out.txt", LockMode::Write)],
            )
            .unwrap();
        let second = reg
            .acquire_many(
                "p1",
                "s1",
                "t1",
                "A",
                vec![req("p1", "out.txt", LockMode::Write)],
            )
            .unwrap();
        assert!(reg
            .acquire_many(
                "p1",
                "s2",
                "t2",
                "B",
                vec![req("p1", "out.txt", LockMode::Write)]
            )
            .is_err());
        drop(second);
        assert!(reg
            .acquire_many(
                "p1",
                "s2",
                "t2",
                "B",
                vec![req("p1", "out.txt", LockMode::Write)]
            )
            .is_err());
        drop(first);
        assert!(reg
            .acquire_many(
                "p1",
                "s2",
                "t2",
                "B",
                vec![req("p1", "out.txt", LockMode::Write)]
            )
            .is_ok());
    }

    #[test]
    fn runtime_write_gate_does_not_cache_failed_turn_hold() {
        let dir = tempdir().unwrap();
        let sandbox = Sandbox::new(dir.path()).unwrap();
        let turn_locks = TurnFileLockStore::new();
        let guards = turn_locks.guards.clone();
        let _ = std::panic::catch_unwind(move || {
            let _guard = guards.lock().unwrap();
            panic!("poison turn lock store");
        });
        let gate = RuntimeWriteGate::new(
            Arc::new(FileLockRegistry::new()),
            turn_locks,
            &sandbox,
            "p1".into(),
            "s1".into(),
            "t1".into(),
            "A".into(),
            false,
            false,
        );

        let first_err = gate.before_write("out.txt").unwrap_err();
        assert!(first_err.contains("poison"));
        let second_err = gate.before_write("out.txt").unwrap_err();
        assert!(second_err.contains("poison"));
    }

    #[test]
    fn normalize_project_path_posix() {
        let sb = sandbox();
        assert_eq!(normalize_project_path(&sb, "a.txt").unwrap(), "a.txt");
    }
}
