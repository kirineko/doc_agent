use serde::Serialize;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;

const MAX_BYTES: u64 = 1_000_000;
const LOG_NAME: &str = "provider-errors.jsonl";
const ROTATED_NAME: &str = "provider-errors.1.jsonl";

pub fn append(data_dir: &Path, record: &impl Serialize) -> io::Result<()> {
    let dir = data_dir.join("logs");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(LOG_NAME);
    if path
        .metadata()
        .map(|m| m.len() > MAX_BYTES)
        .unwrap_or(false)
    {
        let _ = std::fs::rename(&path, dir.join(ROTATED_NAME));
    }
    let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
    writeln!(
        file,
        "{}",
        serde_json::to_string(record).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::tempdir;

    #[test]
    fn appends_two_parseable_lines() {
        let dir = tempdir().unwrap();
        append(dir.path(), &json!({"n": 1})).unwrap();
        append(dir.path(), &json!({"n": 2})).unwrap();
        let text = std::fs::read_to_string(dir.path().join("logs").join(LOG_NAME)).unwrap();
        let lines: Vec<_> = text.lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(lines[0]).unwrap()["n"],
            1
        );
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(lines[1]).unwrap()["n"],
            2
        );
    }

    #[test]
    fn rotates_when_over_1mb() {
        let dir = tempdir().unwrap();
        let logs = dir.path().join("logs");
        std::fs::create_dir_all(&logs).unwrap();
        let path = logs.join(LOG_NAME);
        std::fs::write(&path, "x".repeat(1_000_001)).unwrap();
        append(dir.path(), &json!({"n": 1})).unwrap();
        let rotated = std::fs::read_to_string(logs.join(ROTATED_NAME)).unwrap();
        assert!(rotated.len() > 1_000_000);
        let fresh = std::fs::read_to_string(&path).unwrap();
        assert_eq!(fresh.lines().count(), 1);
        assert!(fresh.contains("\"n\":1"));
    }
}
