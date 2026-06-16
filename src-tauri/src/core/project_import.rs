use crate::core::sandbox::Sandbox;
use std::path::Path;

pub const MAX_IMPORT_FILE_BYTES: u64 = 100 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportConflictStrategy {
    FailIfExists,
    Overwrite,
    Rename,
}

#[derive(Debug, PartialEq, Eq, serde::Serialize)]
pub struct ImportResult {
    pub path: String,
    pub renamed: bool,
}

pub fn validate_import_filename(filename: &str) -> Result<(), String> {
    if filename.is_empty() {
        return Err("filename required".into());
    }
    if filename.contains('/') || filename.contains('\\') {
        return Err("filename must not contain path separators".into());
    }
    if filename.contains("..") {
        return Err("invalid filename".into());
    }
    Ok(())
}

pub fn renamed_import_filename(filename: &str, sandbox: &Sandbox) -> Result<String, String> {
    let path = Path::new(filename);
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("file");
    let ext = path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| format!(".{value}"))
        .unwrap_or_default();

    for n in 1..10_000 {
        let candidate = format!("{stem} ({n}){ext}");
        let target = sandbox
            .resolve_for_write(&candidate)
            .map_err(|e| e.to_string())?;
        if !target.exists() {
            return Ok(candidate);
        }
    }
    Err("could not find available filename".into())
}

pub fn import_project_file(
    sandbox: &Sandbox,
    filename: &str,
    bytes: &[u8],
    strategy: ImportConflictStrategy,
) -> Result<ImportResult, String> {
    validate_import_filename(filename)?;
    if bytes.len() as u64 > MAX_IMPORT_FILE_BYTES {
        return Err(format!(
            "file exceeds {}MB limit",
            MAX_IMPORT_FILE_BYTES / 1024 / 1024
        ));
    }

    let initial_target = sandbox
        .resolve_for_write(filename)
        .map_err(|e| e.to_string())?;
    let (final_name, renamed) = if initial_target.exists() {
        match strategy {
            ImportConflictStrategy::FailIfExists => {
                return Err(format!("file already exists: {filename}"));
            }
            ImportConflictStrategy::Overwrite => (filename.to_string(), false),
            ImportConflictStrategy::Rename => {
                let next = renamed_import_filename(filename, sandbox)?;
                (next, true)
            }
        }
    } else {
        (filename.to_string(), false)
    };

    let target = sandbox
        .resolve_for_write(&final_name)
        .map_err(|e| e.to_string())?;
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&target, bytes).map_err(|e| e.to_string())?;

    Ok(ImportResult {
        path: final_name.replace('\\', "/"),
        renamed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn sandbox(root: &Path) -> Sandbox {
        Sandbox::new(root).expect("sandbox")
    }

    #[test]
    fn rejects_path_separators_in_filename() {
        assert!(validate_import_filename("docs/a.txt").is_err());
        assert!(validate_import_filename("..\\a.txt").is_err());
    }

    #[test]
    fn imports_to_project_root() {
        let dir = tempdir().unwrap();
        let sb = sandbox(dir.path());
        let result = import_project_file(
            &sb,
            "note.txt",
            b"hello",
            ImportConflictStrategy::FailIfExists,
        )
        .unwrap();
        assert_eq!(result.path, "note.txt");
        assert!(!result.renamed);
        assert_eq!(
            fs::read_to_string(dir.path().join("note.txt")).unwrap(),
            "hello"
        );
    }

    #[test]
    fn overwrite_replaces_existing_file() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("note.txt"), b"old").unwrap();
        let sb = sandbox(dir.path());
        import_project_file(&sb, "note.txt", b"new", ImportConflictStrategy::Overwrite).unwrap();
        assert_eq!(
            fs::read_to_string(dir.path().join("note.txt")).unwrap(),
            "new"
        );
    }

    #[test]
    fn rename_increments_until_available() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("note.txt"), b"old").unwrap();
        fs::write(dir.path().join("note (1).txt"), b"one").unwrap();
        let sb = sandbox(dir.path());
        let result =
            import_project_file(&sb, "note.txt", b"new", ImportConflictStrategy::Rename).unwrap();
        assert_eq!(result.path, "note (2).txt");
        assert!(result.renamed);
        assert_eq!(
            fs::read_to_string(dir.path().join("note (2).txt")).unwrap(),
            "new"
        );
    }

    #[test]
    fn fail_if_exists_returns_error() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("note.txt"), b"old").unwrap();
        let sb = sandbox(dir.path());
        let err = import_project_file(
            &sb,
            "note.txt",
            b"new",
            ImportConflictStrategy::FailIfExists,
        )
        .unwrap_err();
        assert!(err.contains("file already exists"));
    }
}
