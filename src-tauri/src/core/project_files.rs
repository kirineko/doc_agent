use serde::Serialize;
use std::fs;
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

#[derive(Debug, Clone, Serialize)]
pub struct ProjectDirEntry {
    pub name: String,
    pub is_dir: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectDirListing {
    pub path: String,
    pub entries: Vec<ProjectDirEntry>,
}

pub const DOCUMENT_EXTENSIONS: &[&str] = &[
    "csv", "xlsx", "xls", "md", "docx", "doc", "pdf", "pptx", "ppt",
];

pub fn text_contains_document_extension(text: &str) -> bool {
    let lower = text.to_lowercase();
    DOCUMENT_EXTENSIONS
        .iter()
        .any(|ext| lower.contains(&format!(".{ext}")))
}

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

pub fn list_project_dir(root: &Path, relative_path: &str) -> Result<ProjectDirListing, String> {
    let sandbox = crate::core::sandbox::Sandbox::new(root).map_err(|e| e.to_string())?;
    let rel = match relative_path.trim() {
        "" => ".",
        other => other,
    };
    let dir = sandbox.resolve(rel).map_err(|e| e.to_string())?;
    if !dir.is_dir() {
        return Err("not a directory".into());
    }

    let mut entries = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if should_skip_name(&name) {
            continue;
        }
        let is_dir = entry.file_type().map_err(|e| e.to_string())?.is_dir();
        entries.push(ProjectDirEntry { name, is_dir });
    }
    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.cmp(&b.name),
    });
    Ok(ProjectDirListing {
        path: rel.to_string(),
        entries,
    })
}

fn should_skip_name(name: &str) -> bool {
    name.starts_with('.') || name == "node_modules" || name == "target" || name.starts_with("~$")
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
    should_skip_name(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn rejects_paths_outside_project_root() {
        let dir = tempdir().unwrap();
        let err = list_project_dir(dir.path(), "..").unwrap_err();
        assert!(err.contains("escapes"), "unexpected error: {err}");
    }

    #[test]
    fn lists_single_directory_level() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("docs")).unwrap();
        fs::write(root.join("docs/report.docx"), b"x").unwrap();
        fs::write(root.join("readme.txt"), b"x").unwrap();
        fs::create_dir_all(root.join("node_modules/pkg")).unwrap();

        let root_list = list_project_dir(root, ".").unwrap();
        let names: Vec<_> = root_list.entries.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"docs"));
        assert!(names.contains(&"readme.txt"));
        assert!(!names.contains(&"node_modules"));

        let docs_list = list_project_dir(root, "docs").unwrap();
        assert_eq!(docs_list.path, "docs");
        assert_eq!(docs_list.entries.len(), 1);
        assert_eq!(docs_list.entries[0].name, "report.docx");
    }

    #[test]
    fn text_contains_document_extension_matches_embedded_paths() {
        assert!(text_contains_document_extension(
            "列出normalized/课程负责人.csv中软件工程"
        ));
        assert!(!text_contains_document_extension("列出目录文件"));
    }

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
