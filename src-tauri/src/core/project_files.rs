use serde::Serialize;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const MAX_DEPTH: usize = 6;
const MAX_ENTRIES: usize = 2000;

#[derive(Debug, Clone, Serialize)]
pub struct ProjectFileEntry {
    pub path: String,
    pub is_dir: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectFileList {
    pub entries: Vec<ProjectFileEntry>,
    pub truncated: bool,
}

const DOCUMENT_EXTENSIONS: &[&str] = &[
    "docx", "xlsx", "pptx", "pdf", "md", "csv", "doc", "xls", "ppt",
];

pub fn is_document_path(rel_path: &str) -> bool {
    Path::new(rel_path)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| {
            DOCUMENT_EXTENSIONS
                .iter()
                .any(|d| ext.eq_ignore_ascii_case(d))
        })
}

/// 按修改时间取最近的可读文档（复用 list_project_files 的忽略规则）。
pub fn recent_document_paths(root: &Path, limit: usize) -> Vec<PathBuf> {
    let file_list = list_project_files(root);
    let mut docs: Vec<(PathBuf, std::time::SystemTime)> = file_list
        .entries
        .iter()
        .filter(|e| !e.is_dir && is_document_path(&e.path))
        .filter_map(|e| {
            let path = root.join(&e.path);
            let mtime = std::fs::metadata(&path).ok()?.modified().ok()?;
            Some((path, mtime))
        })
        .collect();
    docs.sort_by_key(|(_, mtime)| std::cmp::Reverse(*mtime));
    docs.truncate(limit);
    docs.into_iter().map(|(path, _)| path).collect()
}

pub fn list_project_files(root: &Path) -> ProjectFileList {
    let mut entries = Vec::new();
    let mut truncated = false;

    for entry in WalkDir::new(root)
        .max_depth(MAX_DEPTH)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !should_skip_entry(e.path(), root))
        .filter_map(|e| e.ok())
    {
        if entry.path() == root {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(root)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .replace('\\', "/");
        entries.push(ProjectFileEntry {
            path: rel,
            is_dir: entry.file_type().is_dir(),
        });
        if entries.len() >= MAX_ENTRIES {
            truncated = true;
            break;
        }
    }

    entries.sort_by(|a, b| a.path.cmp(&b.path));
    ProjectFileList { entries, truncated }
}

fn should_skip_entry(path: &Path, root: &Path) -> bool {
    if path == root {
        return false;
    }
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if name.starts_with('.') {
        return true;
    }
    if name == "node_modules" || name == "target" {
        return true;
    }
    if name.starts_with("~$") {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn lists_relative_paths_and_skips_ignored_dirs() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("docs")).unwrap();
        fs::write(root.join("docs/report.docx"), b"x").unwrap();
        fs::create_dir_all(root.join("node_modules/pkg")).unwrap();
        fs::write(root.join("node_modules/pkg/a.js"), b"x").unwrap();
        fs::write(root.join("~$temp.docx"), b"x").unwrap();

        let list = list_project_files(root);
        let paths: Vec<_> = list.entries.iter().map(|e| e.path.as_str()).collect();
        assert!(paths.contains(&"docs"));
        assert!(paths.contains(&"docs/report.docx"));
        assert!(!paths.iter().any(|p| p.contains("node_modules")));
        assert!(!paths.iter().any(|p| p.contains("~$")));
    }
}
