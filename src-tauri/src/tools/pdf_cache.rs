use crate::core::cache_paths;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::time::UNIX_EPOCH;

pub use cache_paths::PDF_CACHE_ROOT as CACHE_ROOT;
pub const MANIFEST_FILE: &str = "manifest.json";
pub const DEFAULT_DPI: u32 = 150;
pub const MIN_DPI: u32 = 72;
pub const MAX_DPI: u32 = 300;

pub fn parse_dpi(dpi_arg: Option<u64>) -> Result<u32, String> {
    let dpi = dpi_arg.map(|v| v as u32).unwrap_or(DEFAULT_DPI);
    if dpi < MIN_DPI || dpi > MAX_DPI {
        return Err(format!("dpi must be between {MIN_DPI} and {MAX_DPI}"));
    }
    Ok(dpi)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PageEntry {
    pub index: u32,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RenderManifest {
    pub version: u32,
    pub source_path: String,
    pub source_size: u64,
    pub source_mtime_secs: u64,
    pub dpi: u32,
    pub pages_spec: String,
    pub page_count: u32,
    pub pages: Vec<PageEntry>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFingerprint {
    pub rel_path: String,
    pub size: u64,
    pub mtime_secs: u64,
    pub dpi: u32,
    pub pages_spec: String,
}

pub fn cache_key(fingerprint: &SourceFingerprint) -> String {
    let mut hasher = DefaultHasher::new();
    fingerprint.rel_path.hash(&mut hasher);
    fingerprint.size.hash(&mut hasher);
    fingerprint.mtime_secs.hash(&mut hasher);
    fingerprint.dpi.hash(&mut hasher);
    fingerprint.pages_spec.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

pub fn cache_dir_rel(cache_key: &str) -> String {
    format!("{CACHE_ROOT}/{cache_key}")
}

pub fn page_image_rel(cache_key: &str, page_index: u32) -> String {
    format!(
        "{}/page_{:03}.png",
        cache_dir_rel(cache_key),
        page_index
    )
}

pub fn fingerprint_from_path(
    rel_path: &str,
    abs_path: &Path,
    dpi: u32,
    pages_spec: &str,
) -> Result<SourceFingerprint, String> {
    let metadata = fs::metadata(abs_path).map_err(|e| e.to_string())?;
    let mtime_secs = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    Ok(SourceFingerprint {
        rel_path: rel_path.to_string(),
        size: metadata.len(),
        mtime_secs,
        dpi,
        pages_spec: pages_spec.to_string(),
    })
}

pub fn read_manifest(cache_abs: &Path) -> Result<RenderManifest, String> {
    let raw = fs::read_to_string(cache_abs.join(MANIFEST_FILE)).map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).map_err(|e| format!("invalid manifest: {e}"))
}

pub fn write_manifest(cache_abs: &Path, manifest: &RenderManifest) -> Result<(), String> {
    fs::create_dir_all(cache_abs).map_err(|e| e.to_string())?;
    let raw = serde_json::to_string_pretty(manifest).map_err(|e| e.to_string())?;
    fs::write(cache_abs.join(MANIFEST_FILE), raw).map_err(|e| e.to_string())
}

pub fn manifest_matches(
    manifest: &RenderManifest,
    fingerprint: &SourceFingerprint,
    expected_pages: &[u32],
) -> bool {
    manifest.version == 1
        && manifest.source_path == fingerprint.rel_path
        && manifest.source_size == fingerprint.size
        && manifest.source_mtime_secs == fingerprint.mtime_secs
        && manifest.dpi == fingerprint.dpi
        && manifest.pages_spec == fingerprint.pages_spec
        && manifest.page_count == expected_pages.len() as u32
        && manifest.pages.len() == expected_pages.len()
}

pub fn page_files_exist(cache_abs: &Path, manifest: &RenderManifest) -> bool {
    manifest.pages.iter().all(|entry| {
        let name = format!("page_{:03}.png", entry.index);
        cache_abs.join(name).is_file()
    })
}

pub fn try_cache_hit(
    sandbox_root: &Path,
    fingerprint: &SourceFingerprint,
    expected_pages: &[u32],
) -> Option<RenderManifest> {
    let key = cache_key(fingerprint);
    let cache_abs = sandbox_root.join(cache_dir_rel(&key));
    let manifest = read_manifest(&cache_abs).ok()?;
    if !manifest_matches(&manifest, fingerprint, expected_pages) {
        return None;
    }
    if !page_files_exist(&cache_abs, &manifest) {
        return None;
    }
    Some(manifest)
}

pub fn clear_cache_dir(cache_abs: &Path) -> Result<(), String> {
    if cache_abs.exists() {
        fs::remove_dir_all(cache_abs).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Accept `pages` as string (`"1-4"`, `"1,3"`) or JSON array (`[1,3,5]`).
pub fn normalize_pages_arg(value: Option<&Value>) -> Result<Option<String>, String> {
    let Some(v) = value else {
        return Ok(None);
    };
    if let Some(s) = v.as_str() {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Ok(None);
        }
        return Ok(Some(trimmed.to_string()));
    }
    if let Some(arr) = v.as_array() {
        if arr.is_empty() {
            return Err("pages array must not be empty".into());
        }
        let mut parts = Vec::with_capacity(arr.len());
        for item in arr {
            if let Some(n) = item.as_u64() {
                if n == 0 {
                    return Err("page numbers are 1-based".into());
                }
                parts.push(n.to_string());
            } else if let Some(s) = item.as_str() {
                let trimmed = s.trim();
                if trimmed.is_empty() {
                    return Err("pages array items must not be empty".into());
                }
                parts.push(trimmed.to_string());
            } else {
                return Err("pages array items must be numbers or strings".into());
            }
        }
        return Ok(Some(parts.join(",")));
    }
    Err("pages must be a string (e.g. \"1-4\") or array of page numbers".into())
}

pub fn parse_pages_spec(spec: Option<&str>, total: u32) -> Result<(Vec<u32>, String), String> {
    let normalized = spec
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("all");
    if normalized.eq_ignore_ascii_case("all") {
        let pages: Vec<u32> = (1..=total).collect();
        return Ok((pages, "all".to_string()));
    }

    let mut pages = Vec::new();
    for part in normalized.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some((start, end)) = part.split_once('-') {
            let start: u32 = start
                .trim()
                .parse()
                .map_err(|_| format!("invalid page number: {start}"))?;
            let end: u32 = end
                .trim()
                .parse()
                .map_err(|_| format!("invalid page number: {end}"))?;
            if start == 0 || end == 0 || start > end {
                return Err(format!("invalid page range: {start}-{end}"));
            }
            pages.extend(start..=end);
        } else {
            let page: u32 = part
                .parse()
                .map_err(|_| format!("invalid page number: {part}"))?;
            if page == 0 {
                return Err(format!("invalid page number: {page}"));
            }
            pages.push(page);
        }
    }
    if pages.is_empty() {
        return Err("pages spec is empty".into());
    }
    pages.sort_unstable();
    pages.dedup();
    for page in &pages {
        if *page > total {
            return Err(format!("page {page} out of range (total {total})"));
        }
    }
    Ok((pages, normalized.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_key_changes_when_source_changes() {
        let a = SourceFingerprint {
            rel_path: "a.pdf".into(),
            size: 100,
            mtime_secs: 1,
            dpi: 150,
            pages_spec: "all".into(),
        };
        let mut b = a.clone();
        b.size = 101;
        assert_ne!(cache_key(&a), cache_key(&b));
    }

    #[test]
    fn parse_pages_all_and_ranges() {
        let (pages, spec) = parse_pages_spec(None, 5).unwrap();
        assert_eq!(spec, "all");
        assert_eq!(pages, vec![1, 2, 3, 4, 5]);

        let (pages, _) = parse_pages_spec(Some("1-2,4"), 5).unwrap();
        assert_eq!(pages, vec![1, 2, 4]);
    }

    #[test]
    fn parse_dpi_bounds() {
        assert_eq!(parse_dpi(None).unwrap(), DEFAULT_DPI);
        assert_eq!(parse_dpi(Some(150)).unwrap(), 150);
        assert!(parse_dpi(Some(71)).is_err());
        assert!(parse_dpi(Some(301)).is_err());
    }

    #[test]
    fn normalize_pages_arg_accepts_string_and_array() {
        assert_eq!(
            normalize_pages_arg(None).unwrap(),
            None
        );
        assert_eq!(
            normalize_pages_arg(Some(&serde_json::json!("1-3"))).unwrap(),
            Some("1-3".into())
        );
        assert_eq!(
            normalize_pages_arg(Some(&serde_json::json!([1, 3, 5]))).unwrap(),
            Some("1,3,5".into())
        );
    }

    #[test]
    fn normalize_pages_arg_rejects_invalid() {
        assert!(normalize_pages_arg(Some(&serde_json::json!({}))).is_err());
        assert!(normalize_pages_arg(Some(&serde_json::json!([0]))).is_err());
    }
}
