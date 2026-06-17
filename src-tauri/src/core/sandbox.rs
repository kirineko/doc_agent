use std::path::{Component, Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SandboxError {
    #[error("path escapes project sandbox")]
    EscapesSandbox,
    #[error("path does not exist")]
    NotFound,
    #[error("invalid path")]
    InvalidPath,
    #[error("{0}")]
    Io(String),
}

#[derive(Clone)]
pub struct Sandbox {
    root: PathBuf,
}

impl Sandbox {
    pub fn new(root: impl AsRef<Path>) -> Result<Self, SandboxError> {
        let root = root.as_ref();
        if !root.exists() {
            return Err(SandboxError::NotFound);
        }
        let canonical = root
            .canonicalize()
            .map_err(|e| SandboxError::Io(e.to_string()))?;
        Ok(Self { root: canonical })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn resolve(&self, user_path: &str) -> Result<PathBuf, SandboxError> {
        let path = Path::new(user_path);
        let candidate = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.root.join(path)
        };

        if !candidate.exists() {
            // For write targets, parent must exist inside sandbox
            if let Some(parent) = candidate.parent() {
                if parent.exists() {
                    return self
                        .ensure_within(parent.join(candidate.file_name().unwrap_or_default()));
                }
            }
            return Err(SandboxError::NotFound);
        }

        self.ensure_within(candidate)
    }

    pub fn resolve_for_write(&self, user_path: &str) -> Result<PathBuf, SandboxError> {
        let candidate = self.join_relative(user_path);
        self.ensure_relative_safe(&candidate)?;
        Ok(candidate)
    }

    fn join_relative(&self, user_path: &str) -> PathBuf {
        let path = Path::new(user_path);
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.root.join(path)
        }
    }

    fn ensure_relative_safe(&self, candidate: &Path) -> Result<(), SandboxError> {
        let exists = candidate.exists();
        let base = if exists {
            candidate
                .canonicalize()
                .map_err(|e| SandboxError::Io(e.to_string()))?
        } else {
            candidate.to_path_buf()
        };
        if base.starts_with(&self.root) {
            return Ok(());
        }
        if exists {
            return Err(SandboxError::EscapesSandbox);
        }
        // For not-yet-existing paths, reject parent-dir components relative to root.
        let rel = candidate
            .strip_prefix(&self.root)
            .map_err(|_| SandboxError::EscapesSandbox)?;
        for comp in rel.components() {
            if matches!(comp, Component::ParentDir) {
                return Err(SandboxError::EscapesSandbox);
            }
        }
        Ok(())
    }

    fn ensure_within(&self, path: PathBuf) -> Result<PathBuf, SandboxError> {
        let canonical = path
            .canonicalize()
            .map_err(|e| SandboxError::Io(e.to_string()))?;
        if !canonical.starts_with(&self.root) {
            return Err(SandboxError::EscapesSandbox);
        }
        Ok(canonical)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn rejects_parent_escape() {
        let dir = tempdir().unwrap();
        let sandbox = Sandbox::new(dir.path()).unwrap();
        let outside = dir.path().parent().unwrap().join("outside.txt");
        fs::write(&outside, "secret").unwrap();
        let rel = format!("../{}", outside.file_name().unwrap().to_string_lossy());
        assert_eq!(sandbox.resolve(&rel), Err(SandboxError::EscapesSandbox));
    }

    #[test]
    fn allows_file_inside_root() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("note.txt");
        fs::write(&file, "hello").unwrap();
        let sandbox = Sandbox::new(dir.path()).unwrap();
        let resolved = sandbox.resolve("note.txt").unwrap();
        assert_eq!(resolved, file.canonicalize().unwrap());
    }

    #[test]
    fn resolve_for_write_allows_new_file_in_sandbox() {
        let dir = tempdir().unwrap();
        let sandbox = Sandbox::new(dir.path()).unwrap();
        let target = sandbox.resolve_for_write("notes/new.txt").unwrap();
        assert!(target.starts_with(sandbox.root()));
    }

    #[cfg(unix)]
    #[test]
    fn resolve_for_write_rejects_symlink_escape() {
        let dir = tempdir().unwrap();
        let outside = tempdir().unwrap();
        let outside_file = outside.path().join("secret.txt");
        fs::write(&outside_file, "secret").unwrap();
        std::os::unix::fs::symlink(&outside_file, dir.path().join("linked.txt")).unwrap();

        let sandbox = Sandbox::new(dir.path()).unwrap();
        assert_eq!(
            sandbox.resolve_for_write("linked.txt"),
            Err(SandboxError::EscapesSandbox)
        );
    }
}
