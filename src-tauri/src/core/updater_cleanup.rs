use std::fs::DirEntry;
use std::path::Path;
use std::time::{Duration, SystemTime};

const PRODUCT: &str = "DocAgent";
const MAX_ENTRIES: usize = 512;
const STALE_AGE: Duration = Duration::from_secs(24 * 60 * 60);

pub fn spawn_stale_cleanup() {
    let _ = std::thread::Builder::new()
        .name("updater-cleanup".into())
        .spawn(cleanup_stale_updater_artifacts);
}

pub fn cleanup_stale_updater_artifacts() {
    let cutoff = SystemTime::now()
        .checked_sub(STALE_AGE)
        .unwrap_or(SystemTime::UNIX_EPOCH);
    cleanup_stale_updater_artifacts_in(&std::env::temp_dir(), cutoff);
}

fn cleanup_stale_updater_artifacts_in(temp_dir: &Path, cutoff: SystemTime) {
    let entries = match std::fs::read_dir(temp_dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for (scanned, entry) in entries.flatten().enumerate() {
        if scanned >= MAX_ENTRIES {
            break;
        }

        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();
        if !is_updater_artifact_name(&name) {
            continue;
        }

        let created = match entry_created_at(&entry) {
            Some(time) => time,
            None => continue,
        };
        if !is_stale_updater_entry(created, cutoff) {
            continue;
        }

        let path = entry.path();
        if path.is_dir() {
            let _ = std::fs::remove_dir_all(path);
        } else {
            let _ = std::fs::remove_file(path);
        }
    }
}

/// Matches only known Doc Agent updater / installer artifacts in temp.
///
/// Sources:
/// - `tauri-plugin-updater` temp dir: `DocAgent-{CalVer}-updater-{random}/`
/// - same plugin temp exe: `DocAgent-{CalVer}-installer.exe`
/// - Tauri NSIS bundle / updater download: `DocAgent_{CalVer}_x64-setup.exe`
pub fn is_updater_artifact_name(name: &str) -> bool {
    is_updater_temp_dir(name) || is_updater_temp_installer(name) || is_release_nsis_installer(name)
}

/// `make_temp_dir()` prefix: `{app_name}-{version}-updater-`
fn is_updater_temp_dir(name: &str) -> bool {
    if name.ends_with(".exe") || name.ends_with(".msi") {
        return false;
    }
    let Some(rest) = name.strip_prefix(&format!("{PRODUCT}-")) else {
        return false;
    };
    let Some((version, suffix)) = rest.split_once("-updater-") else {
        return false;
    };
    looks_like_calver(version) && !suffix.is_empty()
}

/// `write_to_temp()` prefix: `{app_name}-{version}-installer.exe`
fn is_updater_temp_installer(name: &str) -> bool {
    let Some(stem) = name.strip_suffix("-installer.exe") else {
        return false;
    };
    let Some(version) = stem.strip_prefix(&format!("{PRODUCT}-")) else {
        return false;
    };
    looks_like_calver(version)
}

/// Tauri bundler NSIS output: `DocAgent_{CalVer}_x64-setup.exe`
fn is_release_nsis_installer(name: &str) -> bool {
    let Some(stem) = name.strip_suffix("_x64-setup.exe") else {
        return false;
    };
    let Some(version) = stem.strip_prefix(&format!("{PRODUCT}_")) else {
        return false;
    };
    looks_like_calver(version)
}

fn looks_like_calver(value: &str) -> bool {
    let mut parts = value.split('.');
    let Some(major) = parts.next() else {
        return false;
    };
    let Some(minor) = parts.next() else {
        return false;
    };
    let Some(patch) = parts.next() else {
        return false;
    };
    if parts.next().is_some() {
        return false;
    }
    [major, minor, patch]
        .iter()
        .all(|part| is_valid_calver_segment(part))
}

fn is_valid_calver_segment(part: &str) -> bool {
    !part.is_empty() && !part.starts_with('0') && part.chars().all(|ch| ch.is_ascii_digit())
}

fn entry_created_at(entry: &DirEntry) -> Option<SystemTime> {
    entry.metadata().ok()?.created().ok()
}

pub fn is_stale_updater_entry(created: SystemTime, cutoff: SystemTime) -> bool {
    created < cutoff
}

#[cfg(test)]
fn cleanup_stale_updater_artifacts_in_with_cutoff(temp_dir: &Path, cutoff: SystemTime) {
    cleanup_stale_updater_artifacts_in(temp_dir, cutoff);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn matches_known_updater_and_installer_names() {
        assert!(is_updater_artifact_name(
            "DocAgent-2026.6.17-updater-abc123"
        ));
        assert!(is_updater_artifact_name("DocAgent-2026.6.17-installer.exe"));
        assert!(is_updater_artifact_name("DocAgent_2026.6.17_x64-setup.exe"));
    }

    #[test]
    fn rejects_loose_or_foreign_names() {
        assert!(!is_updater_artifact_name("random-temp.txt"));
        assert!(!is_updater_artifact_name("DocAgent-2026.6.1-updater"));
        assert!(!is_updater_artifact_name("DocAgent-2026.6.1-setup.exe"));
        assert!(!is_updater_artifact_name(
            "DocAgent-2026.6.1-updater-setup.exe"
        ));
        assert!(!is_updater_artifact_name("DocAgent-2026.6.1.msi"));
        assert!(!is_updater_artifact_name("OtherApp-1.0-updater-setup.exe"));
        assert!(!is_updater_artifact_name(
            "DocAgent-not-a-version-updater-x"
        ));
    }

    #[test]
    fn calver_validation() {
        assert!(looks_like_calver("2026.6.17"));
        assert!(!looks_like_calver("2026.06.17"));
        assert!(!looks_like_calver("1.0.0-extra"));
    }

    #[test]
    fn stale_threshold_uses_cutoff() {
        let cutoff = SystemTime::now() - STALE_AGE;
        let old = cutoff - Duration::from_secs(60);
        let recent = cutoff + Duration::from_secs(60);
        assert!(is_stale_updater_entry(old, cutoff));
        assert!(!is_stale_updater_entry(recent, cutoff));
    }

    #[test]
    fn entry_created_at_reads_birth_time_when_available() {
        if cfg!(target_os = "linux") {
            return;
        }

        let dir = tempdir().unwrap();
        let path = dir.path().join("DocAgent-2026.6.17-updater-test");
        fs::create_dir_all(&path).unwrap();
        let entry = fs::read_dir(dir.path())
            .unwrap()
            .find(|e| e.as_ref().unwrap().file_name() == "DocAgent-2026.6.17-updater-test")
            .unwrap()
            .unwrap();
        let created = entry_created_at(&entry);
        assert!(created.is_some());
    }

    #[test]
    fn cleanup_deletes_stale_matching_entries() {
        let dir = tempdir().unwrap();
        let updater_dir = dir.path().join("DocAgent-2026.6.17-updater-abc");
        fs::create_dir_all(&updater_dir).unwrap();
        let installer_exe = dir.path().join("DocAgent-2026.6.17-installer.exe");
        fs::write(&installer_exe, b"stub").unwrap();
        let stale_setup = dir.path().join("DocAgent_2026.6.17_x64-setup.exe");
        fs::write(&stale_setup, b"stub").unwrap();
        let legacy_hyphen_setup = dir.path().join("DocAgent-2026.6.17-setup.exe");
        fs::write(&legacy_hyphen_setup, b"stub").unwrap();
        let keep = dir.path().join("unrelated.txt");
        fs::write(&keep, b"keep").unwrap();

        let cutoff = SystemTime::now() + Duration::from_secs(3600);
        cleanup_stale_updater_artifacts_in_with_cutoff(dir.path(), cutoff);

        assert!(!updater_dir.exists());
        assert!(!installer_exe.exists());
        assert!(!stale_setup.exists());
        assert!(legacy_hyphen_setup.exists());
        assert!(keep.exists());
    }

    #[test]
    fn cleanup_skips_recent_docagent_installer() {
        let dir = tempdir().unwrap();
        let setup = dir.path().join("DocAgent_2026.6.17_x64-setup.exe");
        fs::write(&setup, b"stub").unwrap();

        let cutoff = SystemTime::now() - Duration::from_secs(3600);
        cleanup_stale_updater_artifacts_in_with_cutoff(dir.path(), cutoff);

        assert!(setup.exists());
    }

    #[test]
    fn cleanup_skips_recent_matching_entries() {
        let dir = tempdir().unwrap();
        let updater_dir = dir.path().join("DocAgent-2026.6.17-updater-abc");
        fs::create_dir_all(&updater_dir).unwrap();

        let cutoff = SystemTime::now() - Duration::from_secs(3600);
        cleanup_stale_updater_artifacts_in_with_cutoff(dir.path(), cutoff);

        assert!(updater_dir.exists());
    }
}
