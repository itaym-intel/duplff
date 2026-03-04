use crate::actions::ActionLog;
use crate::error::{DuplffError, Result};
use std::path::{Path, PathBuf};

/// Get the log directory: ~/.local/share/duplff/logs/
fn log_dir() -> Result<PathBuf> {
    let dir = dirs::data_local_dir()
        .ok_or_else(|| DuplffError::LogError("cannot determine data directory".into()))?
        .join("duplff")
        .join("logs");
    std::fs::create_dir_all(&dir).map_err(|e| DuplffError::LogError(e.to_string()))?;
    Ok(dir)
}

/// Save an action log and return the file path.
pub fn save_action_log(log: &ActionLog) -> Result<PathBuf> {
    let dir = log_dir()?;
    let filename = format!("{}.json", log.timestamp);
    let path = dir.join(filename);
    let json =
        serde_json::to_string_pretty(log).map_err(|e| DuplffError::LogError(e.to_string()))?;
    std::fs::write(&path, json).map_err(|e| DuplffError::LogError(e.to_string()))?;
    Ok(path)
}

/// Load the most recent action log.
pub fn load_latest_log() -> Result<ActionLog> {
    let dir = log_dir()?;
    let mut entries: Vec<_> = std::fs::read_dir(&dir)
        .map_err(|e| DuplffError::LogError(e.to_string()))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
        .collect();
    entries.sort_by_key(|e| std::cmp::Reverse(e.file_name()));
    let latest = entries
        .first()
        .ok_or_else(|| DuplffError::LogError("no action logs found".into()))?;
    load_log(&latest.path())
}

/// Load a specific action log by path.
pub fn load_log(path: &Path) -> Result<ActionLog> {
    let content =
        std::fs::read_to_string(path).map_err(|e| DuplffError::LogError(e.to_string()))?;
    serde_json::from_str(&content).map_err(|e| DuplffError::LogError(e.to_string()))
}

/// List all log file paths, newest first.
pub fn list_logs() -> Result<Vec<PathBuf>> {
    let dir = log_dir()?;
    let mut entries: Vec<_> = std::fs::read_dir(&dir)
        .map_err(|e| DuplffError::LogError(e.to_string()))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
        .map(|e| e.path())
        .collect();
    entries.sort_by(|a, b| b.cmp(a));
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::{ActionRecord, ActionType};

    fn make_test_log() -> ActionLog {
        ActionLog {
            actions: vec![ActionRecord {
                path: "/test/file.txt".into(),
                action: ActionType::Trashed,
            }],
            bytes_reclaimed: 1024,
            timestamp: "1234567890".to_string(),
        }
    }

    #[test]
    fn save_and_load_round_trip() {
        let log = make_test_log();
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("1234567890.json");
        let json = serde_json::to_string_pretty(&log).unwrap();
        std::fs::write(&path, json).unwrap();
        let loaded = load_log(&path).unwrap();
        assert_eq!(loaded.actions.len(), 1);
        assert_eq!(loaded.bytes_reclaimed, 1024);
        assert_eq!(loaded.timestamp, "1234567890");
    }
}
