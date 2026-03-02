// Data models for duplff-core

use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;
use std::time::SystemTime;

/// Configuration for a scan operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    pub roots: Vec<PathBuf>,
    pub extensions: Option<Vec<String>>,
    pub min_size: u64,
    pub max_size: Option<u64>,
    pub priority_paths: Vec<PathBuf>,
    pub follow_symlinks: bool,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            roots: Vec::new(),
            extensions: None,
            min_size: 1,
            max_size: None,
            priority_paths: Vec::new(),
            follow_symlinks: false,
        }
    }
}

/// A discovered file with metadata.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct FileEntry {
    pub path: PathBuf,
    pub size: u64,
    #[serde(with = "system_time_serde")]
    pub modified: SystemTime,
}

impl Ord for FileEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.path.cmp(&other.path)
    }
}

impl PartialOrd for FileEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// A file with its computed BLAKE3 hash.
#[derive(Debug, Clone)]
pub struct HashedFile {
    pub entry: FileEntry,
    pub hash: [u8; 32],
}

/// Why a file was chosen to keep (or not).
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum KeepReason {
    PriorityPath,
    DeepestPath,
    NewestModification,
    LexicographicFirst,
}

impl fmt::Display for KeepReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeepReason::PriorityPath => write!(f, "in priority directory"),
            KeepReason::DeepestPath => write!(f, "deepest path (most specific)"),
            KeepReason::NewestModification => write!(f, "newest modification time"),
            KeepReason::LexicographicFirst => write!(f, "lexicographically first path"),
        }
    }
}

/// A file with its ranking explanation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankedFile {
    pub entry: FileEntry,
    pub reason: KeepReason,
}

/// A group of duplicate files with a keep recommendation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateGroup {
    #[serde(with = "hash_serde")]
    pub hash: [u8; 32],
    pub size: u64,
    pub keep: RankedFile,
    pub duplicates: Vec<RankedFile>,
}

impl DuplicateGroup {
    pub fn wasted_bytes(&self) -> u64 {
        self.size * self.duplicates.len() as u64
    }
}

/// The full result of a deduplication scan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateReport {
    pub groups: Vec<DuplicateGroup>,
    pub total_files_scanned: usize,
    pub total_bytes_scanned: u64,
    pub total_duplicates: usize,
    pub total_wasted_bytes: u64,
}

mod system_time_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    pub fn serialize<S: Serializer>(time: &SystemTime, ser: S) -> Result<S::Ok, S::Error> {
        let duration = time.duration_since(UNIX_EPOCH).unwrap_or(Duration::ZERO);
        duration.as_secs().serialize(ser)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(de: D) -> Result<SystemTime, D::Error> {
        let secs = u64::deserialize(de)?;
        Ok(UNIX_EPOCH + Duration::from_secs(secs))
    }
}

mod hash_serde {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(hash: &[u8; 32], ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(&hex::encode(hash))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(de: D) -> Result<[u8; 32], D::Error> {
        let s = String::deserialize(de)?;
        let bytes = hex::decode(&s).map_err(serde::de::Error::custom)?;
        let arr: [u8; 32] = bytes
            .try_into()
            .map_err(|_| serde::de::Error::custom("invalid hash length"))?;
        Ok(arr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    #[test]
    fn scan_config_default_is_sensible() {
        let config = ScanConfig::default();
        assert_eq!(config.min_size, 1);
        assert!(!config.follow_symlinks);
        assert!(config.roots.is_empty());
        assert!(config.extensions.is_none());
        assert!(config.priority_paths.is_empty());
    }

    #[test]
    fn file_entry_ordering_is_by_path() {
        let a = FileEntry {
            path: "/a/file.txt".into(),
            size: 100,
            modified: SystemTime::UNIX_EPOCH,
        };
        let b = FileEntry {
            path: "/b/file.txt".into(),
            size: 100,
            modified: SystemTime::UNIX_EPOCH,
        };
        assert!(a < b);
    }

    #[test]
    fn duplicate_group_wasted_bytes() {
        let entry = FileEntry {
            path: "/a.txt".into(),
            size: 1000,
            modified: SystemTime::UNIX_EPOCH,
        };
        let group = DuplicateGroup {
            hash: [0u8; 32],
            size: 1000,
            keep: RankedFile {
                entry: entry.clone(),
                reason: KeepReason::LexicographicFirst,
            },
            duplicates: vec![
                RankedFile {
                    entry: entry.clone(),
                    reason: KeepReason::LexicographicFirst,
                },
                RankedFile {
                    entry: entry.clone(),
                    reason: KeepReason::LexicographicFirst,
                },
            ],
        };
        assert_eq!(group.wasted_bytes(), 2000);
    }

    #[test]
    fn keep_reason_display() {
        assert_eq!(
            KeepReason::PriorityPath.to_string(),
            "in priority directory"
        );
        assert_eq!(
            KeepReason::DeepestPath.to_string(),
            "deepest path (most specific)"
        );
        assert_eq!(
            KeepReason::NewestModification.to_string(),
            "newest modification time"
        );
        assert_eq!(
            KeepReason::LexicographicFirst.to_string(),
            "lexicographically first path"
        );
    }

    #[test]
    fn duplicate_report_serializes_to_json() {
        let report = DuplicateReport {
            groups: vec![],
            total_files_scanned: 100,
            total_bytes_scanned: 50000,
            total_duplicates: 0,
            total_wasted_bytes: 0,
        };
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("\"total_files_scanned\":100"));
    }
}
