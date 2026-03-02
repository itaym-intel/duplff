# duplff-core Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement the duplff-core library crate — a reusable Rust library for scanning directories, hashing files with BLAKE3, grouping duplicates, ranking which to keep, and safely removing duplicates via trash.

**Architecture:** Module-based pipeline with shared types. Each pipeline stage (scan, dedupe, hash, rank, act) is an independent module with clear input/output types. A top-level orchestrator chains them. All heavy work is parallelized with rayon. Progress callbacks enable future UI integration.

**Tech Stack:** Rust 1.93 stable, blake3, ignore, rayon, trash, thiserror, serde/serde_json, tempfile (dev)

---

### Task 1: Scaffold Rust Workspace

**Files:**
- Create: `Cargo.toml` (workspace root)
- Create: `crates/duplff-core/Cargo.toml`
- Create: `crates/duplff-core/src/lib.rs`

**Step 1: Create workspace root Cargo.toml**

```toml
[workspace]
members = ["crates/duplff-core"]
resolver = "2"
```

**Step 2: Create duplff-core crate directory and Cargo.toml**

```toml
[package]
name = "duplff-core"
version = "0.1.0"
edition = "2021"
description = "Core duplicate file detection library for duplff"
license = "MIT"

[dependencies]
blake3 = "1"
ignore = "0.4"
rayon = "1"
trash = "5"
thiserror = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[dev-dependencies]
tempfile = "3"
```

**Step 3: Create minimal lib.rs**

```rust
pub mod error;
pub mod models;
pub mod progress;
pub mod scanner;
pub mod hasher;
pub mod deduper;
pub mod ranker;
pub mod actions;
```

Create empty stub files for each module (just `// TODO` or empty) so the crate compiles.

**Step 4: Verify it compiles**

Run: `cargo check`
Expected: compiles with no errors (warnings OK for empty modules)

**Step 5: Commit**

```bash
git add Cargo.toml crates/
git commit -m "feat: scaffold duplff workspace and core crate"
```

---

### Task 2: Error Types

**Files:**
- Create: `crates/duplff-core/src/error.rs`
- Test: inline `#[cfg(test)]` module

**Step 1: Write the failing test**

In `crates/duplff-core/src/error.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn io_error_converts() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "gone");
        let err: DuplffError = io_err.into();
        assert!(matches!(err, DuplffError::Io(_)));
        assert!(err.to_string().contains("gone"));
    }

    #[test]
    fn error_display_is_meaningful() {
        let err = DuplffError::ScanError("bad path".into());
        assert_eq!(err.to_string(), "scan error: bad path");
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p duplff-core error::tests`
Expected: FAIL — `DuplffError` not defined

**Step 3: Write implementation**

```rust
use std::path::PathBuf;
use thiserror::Error;

/// Unified error type for all duplff-core operations.
#[derive(Debug, Error)]
pub enum DuplffError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("scan error: {0}")]
    ScanError(String),

    #[error("hash error: {0}")]
    HashError(String),

    #[error("trash error: {0}")]
    TrashError(String),
}

/// Convenience alias used throughout duplff-core.
pub type Result<T> = std::result::Result<T, DuplffError>;
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p duplff-core error::tests`
Expected: 2 tests PASS

**Step 5: Commit**

```bash
git add crates/duplff-core/src/error.rs
git commit -m "feat: add DuplffError type with thiserror"
```

---

### Task 3: Core Data Models

**Files:**
- Create: `crates/duplff-core/src/models.rs`
- Test: inline `#[cfg(test)]` module

**Step 1: Write the failing test**

In `crates/duplff-core/src/models.rs`:

```rust
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
            keep: RankedFile { entry: entry.clone(), reason: KeepReason::LexicographicFirst },
            duplicates: vec![
                RankedFile { entry: entry.clone(), reason: KeepReason::LexicographicFirst },
                RankedFile { entry: entry.clone(), reason: KeepReason::LexicographicFirst },
            ],
        };
        assert_eq!(group.wasted_bytes(), 2000);
    }

    #[test]
    fn keep_reason_display() {
        assert_eq!(KeepReason::PriorityPath.to_string(), "in priority directory");
        assert_eq!(KeepReason::DeepestPath.to_string(), "deepest path (most specific)");
        assert_eq!(KeepReason::NewestModification.to_string(), "newest modification time");
        assert_eq!(KeepReason::LexicographicFirst.to_string(), "lexicographically first path");
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
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p duplff-core models::tests`
Expected: FAIL — structs not defined

**Step 3: Write implementation**

```rust
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;
use std::time::SystemTime;

/// Configuration for a scan operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    /// One or more root directories to scan.
    pub roots: Vec<PathBuf>,
    /// If set, only include files with these extensions (without dot, e.g. "rs", "py").
    pub extensions: Option<Vec<String>>,
    /// Minimum file size in bytes. Files smaller are skipped. Default: 1.
    pub min_size: u64,
    /// Optional maximum file size in bytes.
    pub max_size: Option<u64>,
    /// Directories whose files are preferred to keep during ranking.
    pub priority_paths: Vec<PathBuf>,
    /// Whether to follow symbolic links. Default: false.
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
    /// Total bytes wasted by duplicates (excludes the kept file).
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
        let arr: [u8; 32] = bytes.try_into().map_err(|_| serde::de::Error::custom("invalid hash length"))?;
        Ok(arr)
    }
}
```

Note: `hash_serde` uses `hex` — add `hex = "0.4"` to `[dependencies]` in `crates/duplff-core/Cargo.toml`.

**Step 4: Run tests**

Run: `cargo test -p duplff-core models::tests`
Expected: 5 tests PASS

**Step 5: Commit**

```bash
git add crates/duplff-core/src/models.rs crates/duplff-core/Cargo.toml
git commit -m "feat: add core data models with serde support"
```

---

### Task 4: Progress Handler Trait

**Files:**
- Create: `crates/duplff-core/src/progress.rs`
- Test: inline `#[cfg(test)]` module

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_progress_does_not_panic() {
        let p = NoopProgress;
        p.on_scan_progress(100);
        p.on_hash_progress(50, 100);
        p.on_complete(10);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p duplff-core progress::tests`
Expected: FAIL

**Step 3: Write implementation**

```rust
/// Trait for receiving progress updates during scanning and hashing.
///
/// Implement this trait to integrate with a UI (TUI or GUI).
/// Use `NoopProgress` for headless or test scenarios.
pub trait ProgressHandler: Send + Sync {
    /// Called periodically during directory scanning.
    fn on_scan_progress(&self, files_found: usize);

    /// Called periodically during hashing.
    fn on_hash_progress(&self, files_hashed: usize, total: usize);

    /// Called when the full pipeline completes.
    fn on_complete(&self, groups_found: usize);
}

/// A no-op progress handler for tests and non-interactive use.
pub struct NoopProgress;

impl ProgressHandler for NoopProgress {
    fn on_scan_progress(&self, _files_found: usize) {}
    fn on_hash_progress(&self, _files_hashed: usize, _total: usize) {}
    fn on_complete(&self, _groups_found: usize) {}
}
```

**Step 4: Run tests**

Run: `cargo test -p duplff-core progress::tests`
Expected: 1 test PASS

**Step 5: Commit**

```bash
git add crates/duplff-core/src/progress.rs
git commit -m "feat: add ProgressHandler trait and NoopProgress"
```

---

### Task 5: Scanner Module

**Files:**
- Create: `crates/duplff-core/src/scanner.rs`
- Test: inline `#[cfg(test)]` module

**Step 1: Write the failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::progress::NoopProgress;
    use std::fs;
    use tempfile::TempDir;

    fn make_test_tree() -> TempDir {
        let dir = TempDir::new().unwrap();
        // Create files of varying sizes
        fs::write(dir.path().join("a.txt"), "hello").unwrap();         // 5 bytes
        fs::write(dir.path().join("b.py"), "world!").unwrap();         // 6 bytes
        fs::create_dir(dir.path().join("sub")).unwrap();
        fs::write(dir.path().join("sub/c.rs"), "fn main() {}").unwrap(); // 13 bytes
        fs::write(dir.path().join("sub/d.txt"), "hi").unwrap();        // 2 bytes
        // Empty file — should be skipped with min_size=1
        fs::write(dir.path().join("empty.txt"), "").unwrap();
        dir
    }

    #[test]
    fn scans_all_files_with_no_filters() {
        let dir = make_test_tree();
        let config = ScanConfig {
            roots: vec![dir.path().to_path_buf()],
            min_size: 1,
            ..ScanConfig::default()
        };
        let files = scan(&config, &NoopProgress).unwrap();
        assert_eq!(files.len(), 4); // excludes empty.txt
    }

    #[test]
    fn filters_by_extension() {
        let dir = make_test_tree();
        let config = ScanConfig {
            roots: vec![dir.path().to_path_buf()],
            extensions: Some(vec!["txt".into()]),
            min_size: 1,
            ..ScanConfig::default()
        };
        let files = scan(&config, &NoopProgress).unwrap();
        // a.txt (5b) and sub/d.txt (2b) — both >=1 byte with .txt extension
        assert_eq!(files.len(), 2);
        assert!(files.iter().all(|f| f.path.extension().unwrap() == "txt"));
    }

    #[test]
    fn filters_by_min_size() {
        let dir = make_test_tree();
        let config = ScanConfig {
            roots: vec![dir.path().to_path_buf()],
            min_size: 5,
            ..ScanConfig::default()
        };
        let files = scan(&config, &NoopProgress).unwrap();
        assert!(files.iter().all(|f| f.size >= 5));
    }

    #[test]
    fn returns_correct_metadata() {
        let dir = make_test_tree();
        let config = ScanConfig {
            roots: vec![dir.path().to_path_buf()],
            min_size: 1,
            ..ScanConfig::default()
        };
        let files = scan(&config, &NoopProgress).unwrap();
        let a = files.iter().find(|f| f.path.file_name().unwrap() == "a.txt").unwrap();
        assert_eq!(a.size, 5);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p duplff-core scanner::tests`
Expected: FAIL — `scan` not defined

**Step 3: Write implementation**

```rust
use crate::error::{DuplffError, Result};
use crate::models::{FileEntry, ScanConfig};
use crate::progress::ProgressHandler;
use ignore::WalkBuilder;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Scan directories according to config, returning matching file entries.
pub fn scan(config: &ScanConfig, progress: &dyn ProgressHandler) -> Result<Vec<FileEntry>> {
    if config.roots.is_empty() {
        return Err(DuplffError::ScanError("no root directories specified".into()));
    }

    let mut builder = WalkBuilder::new(&config.roots[0]);
    for root in &config.roots[1..] {
        builder.add(root);
    }
    builder
        .follow_links(config.follow_symlinks)
        .hidden(false) // don't skip hidden files (let gitignore handle it)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true);

    let counter = AtomicUsize::new(0);
    let mut files = Vec::new();

    for result in builder.build() {
        let entry = result.map_err(|e| DuplffError::ScanError(e.to_string()))?;

        // Skip directories
        let file_type = match entry.file_type() {
            Some(ft) if ft.is_file() => ft,
            _ => continue,
        };

        let metadata = entry.metadata().map_err(|e| DuplffError::ScanError(e.to_string()))?;
        let size = metadata.len();

        // Apply size filter
        if size < config.min_size {
            continue;
        }
        if let Some(max) = config.max_size {
            if size > max {
                continue;
            }
        }

        // Apply extension filter
        if let Some(ref exts) = config.extensions {
            let file_ext = entry.path().extension().and_then(|e| e.to_str());
            match file_ext {
                Some(ext) if exts.iter().any(|e| e.eq_ignore_ascii_case(ext)) => {}
                _ => continue,
            }
        }

        let modified = metadata.modified().unwrap_or(std::time::UNIX_EPOCH);
        files.push(FileEntry {
            path: entry.into_path(),
            size,
            modified,
        });

        let count = counter.fetch_add(1, Ordering::Relaxed) + 1;
        if count % 1000 == 0 {
            progress.on_scan_progress(count);
        }
    }

    let total = files.len();
    progress.on_scan_progress(total);
    Ok(files)
}
```

**Step 4: Run tests**

Run: `cargo test -p duplff-core scanner::tests`
Expected: 4 tests PASS

**Step 5: Commit**

```bash
git add crates/duplff-core/src/scanner.rs
git commit -m "feat: add directory scanner with ignore crate"
```

---

### Task 6: Hasher Module

**Files:**
- Create: `crates/duplff-core/src/hasher.rs`
- Test: inline `#[cfg(test)]` module

**Step 1: Write the failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn partial_hash_of_small_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("small.txt");
        fs::write(&path, "hello world").unwrap();
        let hash = partial_hash(&path).unwrap();
        // Same content must produce same hash
        let hash2 = partial_hash(&path).unwrap();
        assert_eq!(hash, hash2);
    }

    #[test]
    fn partial_hash_differs_for_different_content() {
        let dir = TempDir::new().unwrap();
        let a = dir.path().join("a.txt");
        let b = dir.path().join("b.txt");
        fs::write(&a, "hello").unwrap();
        fs::write(&b, "world").unwrap();
        assert_ne!(partial_hash(&a).unwrap(), partial_hash(&b).unwrap());
    }

    #[test]
    fn full_hash_matches_blake3_reference() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.txt");
        let content = b"deterministic content for hashing";
        fs::write(&path, content).unwrap();
        let hash = full_hash(&path).unwrap();
        let expected = blake3::hash(content);
        assert_eq!(hash, *expected.as_bytes());
    }

    #[test]
    fn full_hash_of_large_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("large.bin");
        // 1MB of repeated bytes
        let content = vec![0xABu8; 1024 * 1024];
        fs::write(&path, &content).unwrap();
        let hash = full_hash(&path).unwrap();
        let expected = blake3::hash(&content);
        assert_eq!(hash, *expected.as_bytes());
    }

    #[test]
    fn partial_hash_same_for_files_sharing_first_4kb() {
        let dir = TempDir::new().unwrap();
        let a = dir.path().join("a.bin");
        let b = dir.path().join("b.bin");
        // Same first 4KB, different after
        let mut content_a = vec![0u8; 8192];
        let mut content_b = vec![0u8; 8192];
        content_b[4096] = 0xFF; // differ after first 4KB
        fs::write(&a, &content_a).unwrap();
        fs::write(&b, &content_b).unwrap();
        // Partial hashes should be equal (only first 4KB)
        assert_eq!(partial_hash(&a).unwrap(), partial_hash(&b).unwrap());
        // Full hashes should differ
        assert_ne!(full_hash(&a).unwrap(), full_hash(&b).unwrap());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p duplff-core hasher::tests`
Expected: FAIL — `partial_hash`, `full_hash` not defined

**Step 3: Write implementation**

```rust
use crate::error::{DuplffError, Result};
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Size of the partial hash sample: first 4KB.
const PARTIAL_HASH_SIZE: usize = 4096;

/// Buffer size for streaming full-file hashing: 128KB.
const HASH_BUFFER_SIZE: usize = 128 * 1024;

/// Compute a BLAKE3 hash of the first 4KB of a file.
///
/// For files smaller than 4KB, hashes the entire content.
/// This is used as a cheap pre-filter before full hashing.
pub fn partial_hash(path: &Path) -> Result<[u8; 32]> {
    let mut file = File::open(path).map_err(|e| DuplffError::HashError(format!("{}: {}", path.display(), e)))?;
    let mut buf = vec![0u8; PARTIAL_HASH_SIZE];
    let n = file.read(&mut buf).map_err(|e| DuplffError::HashError(format!("{}: {}", path.display(), e)))?;
    Ok(*blake3::hash(&buf[..n]).as_bytes())
}

/// Compute a full BLAKE3 hash of a file's entire content.
///
/// Uses 128KB buffered reads for throughput.
pub fn full_hash(path: &Path) -> Result<[u8; 32]> {
    let mut file = File::open(path).map_err(|e| DuplffError::HashError(format!("{}: {}", path.display(), e)))?;
    let mut hasher = blake3::Hasher::new();
    let mut buf = vec![0u8; HASH_BUFFER_SIZE];
    loop {
        let n = file.read(&mut buf).map_err(|e| DuplffError::HashError(format!("{}: {}", path.display(), e)))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(*hasher.finalize().as_bytes())
}
```

**Step 4: Run tests**

Run: `cargo test -p duplff-core hasher::tests`
Expected: 5 tests PASS

**Step 5: Commit**

```bash
git add crates/duplff-core/src/hasher.rs
git commit -m "feat: add BLAKE3 partial and full hash functions"
```

---

### Task 7: Deduper Module

**Files:**
- Create: `crates/duplff-core/src/deduper.rs`
- Test: inline `#[cfg(test)]` module

**Step 1: Write the failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::FileEntry;
    use crate::progress::NoopProgress;
    use std::fs;
    use std::time::SystemTime;
    use tempfile::TempDir;

    #[test]
    fn group_by_size_filters_unique_sizes() {
        let files = vec![
            FileEntry { path: "/a".into(), size: 100, modified: SystemTime::UNIX_EPOCH },
            FileEntry { path: "/b".into(), size: 100, modified: SystemTime::UNIX_EPOCH },
            FileEntry { path: "/c".into(), size: 200, modified: SystemTime::UNIX_EPOCH }, // unique
        ];
        let groups = group_by_size(files);
        assert_eq!(groups.len(), 1); // only the size=100 group
        assert_eq!(groups[0].len(), 2);
    }

    #[test]
    fn group_by_size_empty_input() {
        let groups = group_by_size(vec![]);
        assert!(groups.is_empty());
    }

    #[test]
    fn group_by_size_all_unique() {
        let files = vec![
            FileEntry { path: "/a".into(), size: 100, modified: SystemTime::UNIX_EPOCH },
            FileEntry { path: "/b".into(), size: 200, modified: SystemTime::UNIX_EPOCH },
        ];
        let groups = group_by_size(files);
        assert!(groups.is_empty());
    }

    #[test]
    fn find_duplicates_end_to_end() {
        let dir = TempDir::new().unwrap();
        // Two identical files
        fs::write(dir.path().join("a.txt"), "duplicate content").unwrap();
        fs::write(dir.path().join("b.txt"), "duplicate content").unwrap();
        // Same size but different content
        fs::write(dir.path().join("c.txt"), "different_content").unwrap(); // same length!
        // Different size — will be filtered by size grouping if unique
        fs::write(dir.path().join("d.txt"), "unique").unwrap();

        let files: Vec<FileEntry> = ["a.txt", "b.txt", "c.txt", "d.txt"]
            .iter()
            .map(|name| {
                let path = dir.path().join(name);
                let meta = fs::metadata(&path).unwrap();
                FileEntry {
                    path,
                    size: meta.len(),
                    modified: meta.modified().unwrap(),
                }
            })
            .collect();

        let groups = find_duplicate_groups(files, &NoopProgress).unwrap();
        // a.txt and b.txt are duplicates; c.txt has same size but different content
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].len(), 2);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p duplff-core deduper::tests`
Expected: FAIL

**Step 3: Write implementation**

```rust
use crate::error::Result;
use crate::hasher;
use crate::models::{FileEntry, HashedFile};
use crate::progress::ProgressHandler;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Group files by size, discarding groups with only one file.
pub fn group_by_size(files: Vec<FileEntry>) -> Vec<Vec<FileEntry>> {
    let mut size_map: HashMap<u64, Vec<FileEntry>> = HashMap::new();
    for file in files {
        size_map.entry(file.size).or_default().push(file);
    }
    size_map.into_values().filter(|g| g.len() > 1).collect()
}

/// Run the full deduplication pipeline: size grouping -> partial hash -> full hash.
///
/// Returns groups of files that are byte-identical (same size + same full BLAKE3 hash).
/// Each inner Vec has at least 2 files.
pub fn find_duplicate_groups(
    files: Vec<FileEntry>,
    progress: &dyn ProgressHandler,
) -> Result<Vec<Vec<HashedFile>>> {
    // Stage 1: Group by size
    let size_groups = group_by_size(files);

    // Flatten for hashing
    let candidates: Vec<FileEntry> = size_groups.into_iter().flatten().collect();
    let total = candidates.len();

    // Stage 2: Partial hash
    let hashed_count = AtomicUsize::new(0);
    let partial_results: Vec<Result<(FileEntry, [u8; 32])>> = candidates
        .into_par_iter()
        .map(|entry| {
            let hash = hasher::partial_hash(&entry.path)?;
            let count = hashed_count.fetch_add(1, Ordering::Relaxed) + 1;
            if count % 100 == 0 {
                progress.on_hash_progress(count, total);
            }
            Ok((entry, hash))
        })
        .collect();

    // Collect results, skip errors (file may have been deleted mid-scan)
    let mut partial_map: HashMap<(u64, [u8; 32]), Vec<FileEntry>> = HashMap::new();
    for result in partial_results {
        if let Ok((entry, hash)) = result {
            partial_map.entry((entry.size, hash)).or_default().push(entry);
        }
    }

    // Drop unique partial hashes
    let partial_groups: Vec<Vec<FileEntry>> = partial_map
        .into_values()
        .filter(|g| g.len() > 1)
        .collect();

    let candidates: Vec<FileEntry> = partial_groups.into_iter().flatten().collect();
    let total = candidates.len();

    // Stage 3: Full hash
    let hashed_count = AtomicUsize::new(0);
    let full_results: Vec<Result<HashedFile>> = candidates
        .into_par_iter()
        .map(|entry| {
            let hash = hasher::full_hash(&entry.path)?;
            let count = hashed_count.fetch_add(1, Ordering::Relaxed) + 1;
            if count % 100 == 0 {
                progress.on_hash_progress(count, total);
            }
            Ok(HashedFile { entry, hash })
        })
        .collect();

    // Group by full hash
    let mut full_map: HashMap<(u64, [u8; 32]), Vec<HashedFile>> = HashMap::new();
    for result in full_results {
        if let Ok(hf) = result {
            full_map.entry((hf.entry.size, hf.hash)).or_default().push(hf);
        }
    }

    // Keep only groups with 2+ files
    let groups: Vec<Vec<HashedFile>> = full_map
        .into_values()
        .filter(|g| g.len() > 1)
        .collect();

    Ok(groups)
}
```

**Step 4: Run tests**

Run: `cargo test -p duplff-core deduper::tests`
Expected: 4 tests PASS

**Step 5: Commit**

```bash
git add crates/duplff-core/src/deduper.rs
git commit -m "feat: add deduper with size grouping and two-stage hashing"
```

---

### Task 8: Ranker Module

**Files:**
- Create: `crates/duplff-core/src/ranker.rs`
- Test: inline `#[cfg(test)]` module

**Step 1: Write the failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{FileEntry, HashedFile};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    fn make_hashed(path: &str, modified: SystemTime) -> HashedFile {
        HashedFile {
            entry: FileEntry {
                path: path.into(),
                size: 100,
                modified,
            },
            hash: [0u8; 32],
        }
    }

    #[test]
    fn priority_path_wins() {
        let group = vec![
            make_hashed("/tmp/copy.txt", UNIX_EPOCH + Duration::from_secs(200)),
            make_hashed("/src/original.txt", UNIX_EPOCH + Duration::from_secs(100)),
        ];
        let priority = vec![std::path::PathBuf::from("/src")];
        let ranked = rank_group(group, &priority);
        assert_eq!(ranked.keep.entry.path.to_str().unwrap(), "/src/original.txt");
        assert_eq!(ranked.keep.reason, KeepReason::PriorityPath);
    }

    #[test]
    fn deepest_path_wins_when_no_priority() {
        let group = vec![
            make_hashed("/a/file.txt", UNIX_EPOCH),
            make_hashed("/a/b/c/file.txt", UNIX_EPOCH),
        ];
        let ranked = rank_group(group, &[]);
        assert_eq!(ranked.keep.entry.path.to_str().unwrap(), "/a/b/c/file.txt");
        assert_eq!(ranked.keep.reason, KeepReason::DeepestPath);
    }

    #[test]
    fn newest_mtime_wins_when_same_depth() {
        let group = vec![
            make_hashed("/a/old.txt", UNIX_EPOCH + Duration::from_secs(100)),
            make_hashed("/b/new.txt", UNIX_EPOCH + Duration::from_secs(200)),
        ];
        let ranked = rank_group(group, &[]);
        assert_eq!(ranked.keep.entry.path.to_str().unwrap(), "/b/new.txt");
        assert_eq!(ranked.keep.reason, KeepReason::NewestModification);
    }

    #[test]
    fn lexicographic_tiebreaker() {
        let t = UNIX_EPOCH;
        let group = vec![
            make_hashed("/z/file.txt", t),
            make_hashed("/a/file.txt", t),
        ];
        let ranked = rank_group(group, &[]);
        assert_eq!(ranked.keep.entry.path.to_str().unwrap(), "/a/file.txt");
        assert_eq!(ranked.keep.reason, KeepReason::LexicographicFirst);
    }

    #[test]
    fn deterministic_across_input_order() {
        let t = UNIX_EPOCH;
        let group1 = vec![
            make_hashed("/b.txt", t),
            make_hashed("/a.txt", t),
        ];
        let group2 = vec![
            make_hashed("/a.txt", t),
            make_hashed("/b.txt", t),
        ];
        let r1 = rank_group(group1, &[]);
        let r2 = rank_group(group2, &[]);
        assert_eq!(r1.keep.entry.path, r2.keep.entry.path);
    }

    #[test]
    fn rank_groups_produces_duplicate_groups() {
        let groups = vec![
            vec![
                make_hashed("/a.txt", UNIX_EPOCH),
                make_hashed("/b.txt", UNIX_EPOCH),
            ],
        ];
        let result = rank_groups(groups, &[]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].duplicates.len(), 1);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p duplff-core ranker::tests`
Expected: FAIL

**Step 3: Write implementation**

```rust
use crate::models::{DuplicateGroup, HashedFile, KeepReason, RankedFile};
use std::path::Path;

/// Rank a single group of identical files and produce a DuplicateGroup.
pub fn rank_group(files: Vec<HashedFile>, priority_paths: &[impl AsRef<Path>]) -> DuplicateGroup {
    assert!(files.len() >= 2, "rank_group requires at least 2 files");

    let hash = files[0].hash;
    let size = files[0].entry.size;

    // Score each file: (priority_match, path_depth, mtime_secs, reverse_path_for_lex)
    let mut scored: Vec<(HashedFile, KeepReason, (bool, usize, u64, std::cmp::Reverse<std::path::PathBuf>))> = files
        .into_iter()
        .map(|hf| {
            let in_priority = priority_paths.iter().any(|p| hf.entry.path.starts_with(p.as_ref()));
            let depth = hf.entry.path.components().count();
            let mtime = hf.entry.modified
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let score = (in_priority, depth, mtime, std::cmp::Reverse(hf.entry.path.clone()));

            // Determine the reason this file would be kept
            let reason = if in_priority {
                KeepReason::PriorityPath
            } else {
                // Reason will be refined after sorting
                KeepReason::LexicographicFirst
            };

            (hf, reason, score)
        })
        .collect();

    // Sort descending by score (highest = best candidate to keep)
    scored.sort_by(|a, b| b.2.cmp(&a.2));

    // Determine the actual reason for the winner
    let winner_score = &scored[0].2;
    let runner_up_score = &scored[1].2;
    let reason = if winner_score.0 && !runner_up_score.0 {
        KeepReason::PriorityPath
    } else if winner_score.1 != runner_up_score.1 {
        KeepReason::DeepestPath
    } else if winner_score.2 != runner_up_score.2 {
        KeepReason::NewestModification
    } else {
        KeepReason::LexicographicFirst
    };

    let mut iter = scored.into_iter();
    let (winner_hf, _, _) = iter.next().unwrap();
    let keep = RankedFile {
        entry: winner_hf.entry,
        reason,
    };

    let duplicates: Vec<RankedFile> = iter
        .map(|(hf, _, _)| RankedFile {
            entry: hf.entry,
            reason: KeepReason::LexicographicFirst, // duplicates get a generic reason
        })
        .collect();

    DuplicateGroup {
        hash,
        size,
        keep,
        duplicates,
    }
}

/// Rank all duplicate groups.
pub fn rank_groups(
    groups: Vec<Vec<HashedFile>>,
    priority_paths: &[impl AsRef<Path>],
) -> Vec<DuplicateGroup> {
    groups
        .into_iter()
        .map(|g| rank_group(g, priority_paths))
        .collect()
}
```

**Step 4: Run tests**

Run: `cargo test -p duplff-core ranker::tests`
Expected: 6 tests PASS

**Step 5: Commit**

```bash
git add crates/duplff-core/src/ranker.rs
git commit -m "feat: add deterministic ranker with priority paths and explainable reasons"
```

---

### Task 9: Actions Module

**Files:**
- Create: `crates/duplff-core/src/actions.rs`
- Test: inline `#[cfg(test)]` module

**Step 1: Write the failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;
    use std::time::SystemTime;

    fn make_group(paths: &[&str]) -> DuplicateGroup {
        let keep = RankedFile {
            entry: FileEntry { path: paths[0].into(), size: 100, modified: SystemTime::UNIX_EPOCH },
            reason: KeepReason::LexicographicFirst,
        };
        let duplicates: Vec<RankedFile> = paths[1..]
            .iter()
            .map(|p| RankedFile {
                entry: FileEntry { path: (*p).into(), size: 100, modified: SystemTime::UNIX_EPOCH },
                reason: KeepReason::LexicographicFirst,
            })
            .collect();
        DuplicateGroup {
            hash: [0u8; 32],
            size: 100,
            keep,
            duplicates,
        }
    }

    #[test]
    fn dry_run_lists_files_to_delete() {
        let groups = vec![make_group(&["/keep.txt", "/delete1.txt", "/delete2.txt"])];
        let plan = dry_run(&groups);
        assert_eq!(plan.files_to_delete.len(), 2);
        assert_eq!(plan.bytes_to_reclaim, 200);
        assert!(plan.files_to_delete.contains(&std::path::PathBuf::from("/delete1.txt")));
    }

    #[test]
    fn dry_run_never_includes_keep_file() {
        let groups = vec![make_group(&["/keep.txt", "/dup.txt"])];
        let plan = dry_run(&groups);
        assert!(!plan.files_to_delete.contains(&std::path::PathBuf::from("/keep.txt")));
    }

    #[test]
    fn trash_duplicates_moves_files() {
        use std::fs;
        use tempfile::TempDir;

        let dir = TempDir::new().unwrap();
        let keep = dir.path().join("keep.txt");
        let dup = dir.path().join("dup.txt");
        fs::write(&keep, "content").unwrap();
        fs::write(&dup, "content").unwrap();

        let groups = vec![DuplicateGroup {
            hash: [0u8; 32],
            size: 7,
            keep: RankedFile {
                entry: FileEntry { path: keep.clone(), size: 7, modified: SystemTime::UNIX_EPOCH },
                reason: KeepReason::LexicographicFirst,
            },
            duplicates: vec![RankedFile {
                entry: FileEntry { path: dup.clone(), size: 7, modified: SystemTime::UNIX_EPOCH },
                reason: KeepReason::LexicographicFirst,
            }],
        }];

        let log = trash_duplicates(&groups).unwrap();
        assert!(keep.exists(), "keep file should still exist");
        assert!(!dup.exists(), "duplicate should be trashed");
        assert_eq!(log.actions.len(), 1);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p duplff-core actions::tests`
Expected: FAIL

**Step 3: Write implementation**

```rust
use crate::error::{DuplffError, Result};
use crate::models::DuplicateGroup;
use std::path::PathBuf;

/// Plan produced by a dry run — describes what would be deleted.
#[derive(Debug, Clone)]
pub struct ActionPlan {
    pub files_to_delete: Vec<PathBuf>,
    pub bytes_to_reclaim: u64,
}

/// Record of a single action taken.
#[derive(Debug, Clone)]
pub struct ActionRecord {
    pub path: PathBuf,
    pub action: ActionType,
}

#[derive(Debug, Clone)]
pub enum ActionType {
    Trashed,
}

/// Log of all actions taken during a trash operation.
#[derive(Debug, Clone)]
pub struct ActionLog {
    pub actions: Vec<ActionRecord>,
    pub bytes_reclaimed: u64,
}

/// Produce a dry-run plan: which files would be deleted and how much space reclaimed.
pub fn dry_run(groups: &[DuplicateGroup]) -> ActionPlan {
    let mut files_to_delete = Vec::new();
    let mut bytes_to_reclaim = 0u64;

    for group in groups {
        for dup in &group.duplicates {
            files_to_delete.push(dup.entry.path.clone());
            bytes_to_reclaim += group.size;
        }
    }

    ActionPlan {
        files_to_delete,
        bytes_to_reclaim,
    }
}

/// Move duplicate files to the OS trash. The keep file is never touched.
pub fn trash_duplicates(groups: &[DuplicateGroup]) -> Result<ActionLog> {
    let mut actions = Vec::new();
    let mut bytes_reclaimed = 0u64;

    for group in groups {
        for dup in &group.duplicates {
            trash::delete(&dup.entry.path).map_err(|e| {
                DuplffError::TrashError(format!("{}: {}", dup.entry.path.display(), e))
            })?;
            actions.push(ActionRecord {
                path: dup.entry.path.clone(),
                action: ActionType::Trashed,
            });
            bytes_reclaimed += group.size;
        }
    }

    Ok(ActionLog {
        actions,
        bytes_reclaimed,
    })
}
```

**Step 4: Run tests**

Run: `cargo test -p duplff-core actions::tests`
Expected: 3 tests PASS (the trash test may need a desktop trash service — if it fails in CI, mark it `#[ignore]`)

**Step 5: Commit**

```bash
git add crates/duplff-core/src/actions.rs
git commit -m "feat: add dry-run and trash-based deletion actions"
```

---

### Task 10: Orchestrator (lib.rs) and Integration Tests

**Files:**
- Modify: `crates/duplff-core/src/lib.rs`
- Create: `crates/duplff-core/tests/integration.rs`

**Step 1: Write the failing integration test**

In `crates/duplff-core/tests/integration.rs`:

```rust
use duplff_core::models::ScanConfig;
use duplff_core::progress::NoopProgress;
use duplff_core::{find_duplicates, actions};
use std::fs;
use tempfile::TempDir;

#[test]
fn full_pipeline_finds_exact_duplicates() {
    let dir = TempDir::new().unwrap();

    // Group 1: two identical files
    fs::write(dir.path().join("original.py"), "def hello(): pass").unwrap();
    fs::write(dir.path().join("copy.py"), "def hello(): pass").unwrap();

    // Group 2: three identical files
    let big_content = "x".repeat(10_000);
    fs::create_dir(dir.path().join("src")).unwrap();
    fs::write(dir.path().join("src/main.rs"), &big_content).unwrap();
    fs::write(dir.path().join("backup.rs"), &big_content).unwrap();
    fs::write(dir.path().join("old.rs"), &big_content).unwrap();

    // Not a duplicate: same size as group 1 but different content
    fs::write(dir.path().join("different.py"), "def world(): pass").unwrap();

    // Unique file
    fs::write(dir.path().join("unique.txt"), "only one of me here!!!!").unwrap();

    let config = ScanConfig {
        roots: vec![dir.path().to_path_buf()],
        min_size: 1,
        ..ScanConfig::default()
    };

    let report = find_duplicates(&config, &NoopProgress).unwrap();

    assert_eq!(report.groups.len(), 2);
    assert_eq!(report.total_duplicates, 3); // 1 from group1 + 2 from group2
    assert!(report.total_wasted_bytes > 0);
}

#[test]
fn full_pipeline_with_extension_filter() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("a.py"), "same content").unwrap();
    fs::write(dir.path().join("b.py"), "same content").unwrap();
    fs::write(dir.path().join("c.txt"), "same content").unwrap(); // should be excluded

    let config = ScanConfig {
        roots: vec![dir.path().to_path_buf()],
        extensions: Some(vec!["py".into()]),
        min_size: 1,
        ..ScanConfig::default()
    };

    let report = find_duplicates(&config, &NoopProgress).unwrap();
    assert_eq!(report.groups.len(), 1);
    assert_eq!(report.groups[0].duplicates.len(), 1);
}

#[test]
fn full_pipeline_with_priority_paths() {
    let dir = TempDir::new().unwrap();

    fs::create_dir(dir.path().join("src")).unwrap();
    fs::write(dir.path().join("src/main.rs"), "fn main() {}").unwrap();
    fs::write(dir.path().join("copy.rs"), "fn main() {}").unwrap();

    let config = ScanConfig {
        roots: vec![dir.path().to_path_buf()],
        priority_paths: vec![dir.path().join("src")],
        min_size: 1,
        ..ScanConfig::default()
    };

    let report = find_duplicates(&config, &NoopProgress).unwrap();
    assert_eq!(report.groups.len(), 1);
    assert!(report.groups[0].keep.entry.path.starts_with(dir.path().join("src")));
}

#[test]
fn dry_run_does_not_delete_files() {
    let dir = TempDir::new().unwrap();

    fs::write(dir.path().join("a.txt"), "duplicate").unwrap();
    fs::write(dir.path().join("b.txt"), "duplicate").unwrap();

    let config = ScanConfig {
        roots: vec![dir.path().to_path_buf()],
        min_size: 1,
        ..ScanConfig::default()
    };

    let report = find_duplicates(&config, &NoopProgress).unwrap();
    let plan = actions::dry_run(&report.groups);

    assert_eq!(plan.files_to_delete.len(), 1);
    // Both files should still exist
    assert!(dir.path().join("a.txt").exists());
    assert!(dir.path().join("b.txt").exists());
}

#[test]
fn same_size_different_content_not_grouped() {
    let dir = TempDir::new().unwrap();

    // 10 bytes each, different content
    fs::write(dir.path().join("a.txt"), "aaaaaaaaaa").unwrap();
    fs::write(dir.path().join("b.txt"), "bbbbbbbbbb").unwrap();

    let config = ScanConfig {
        roots: vec![dir.path().to_path_buf()],
        min_size: 1,
        ..ScanConfig::default()
    };

    let report = find_duplicates(&config, &NoopProgress).unwrap();
    assert_eq!(report.groups.len(), 0);
}

#[test]
fn report_serializes_to_json() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("a.txt"), "dup").unwrap();
    fs::write(dir.path().join("b.txt"), "dup").unwrap();

    let config = ScanConfig {
        roots: vec![dir.path().to_path_buf()],
        min_size: 1,
        ..ScanConfig::default()
    };

    let report = find_duplicates(&config, &NoopProgress).unwrap();
    let json = serde_json::to_string_pretty(&report).unwrap();
    assert!(json.contains("total_files_scanned"));
    assert!(json.contains("total_duplicates"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p duplff-core --test integration`
Expected: FAIL — `find_duplicates` not defined

**Step 3: Write the orchestrator in lib.rs**

```rust
pub mod error;
pub mod models;
pub mod progress;
pub mod scanner;
pub mod hasher;
pub mod deduper;
pub mod ranker;
pub mod actions;

use error::Result;
use models::{DuplicateReport, ScanConfig};
use progress::ProgressHandler;

/// Run the full duplicate-finding pipeline.
///
/// Scans directories, groups by size, hashes candidates (partial then full),
/// ranks each group, and returns a complete report.
pub fn find_duplicates(
    config: &ScanConfig,
    progress: &dyn ProgressHandler,
) -> Result<DuplicateReport> {
    // 1. Scan directories
    let files = scanner::scan(config, progress)?;
    let total_files_scanned = files.len();
    let total_bytes_scanned: u64 = files.iter().map(|f| f.size).sum();

    // 2. Group by size, partial hash, full hash
    let duplicate_groups = deduper::find_duplicate_groups(files, progress)?;

    // 3. Rank groups
    let groups = ranker::rank_groups(duplicate_groups, &config.priority_paths);

    // 4. Compute stats
    let total_duplicates: usize = groups.iter().map(|g| g.duplicates.len()).sum();
    let total_wasted_bytes: u64 = groups.iter().map(|g| g.wasted_bytes()).sum();

    progress.on_complete(groups.len());

    Ok(DuplicateReport {
        groups,
        total_files_scanned,
        total_bytes_scanned,
        total_duplicates,
        total_wasted_bytes,
    })
}
```

**Step 4: Run all tests**

Run: `cargo test -p duplff-core`
Expected: ALL tests PASS (unit + integration)

**Step 5: Commit**

```bash
git add crates/duplff-core/src/lib.rs crates/duplff-core/tests/integration.rs
git commit -m "feat: add orchestrator and integration tests for full pipeline"
```

---

### Task 11: Final Verification

**Step 1: Run full test suite with verbose output**

Run: `cargo test -p duplff-core -- --nocapture`
Expected: All tests pass

**Step 2: Run clippy**

Run: `cargo clippy -p duplff-core -- -D warnings`
Expected: No warnings

**Step 3: Check formatting**

Run: `cargo fmt -p duplff-core -- --check`
Expected: No formatting issues (fix if needed with `cargo fmt`)

**Step 4: Commit any fixes**

```bash
git add -A
git commit -m "chore: fix clippy warnings and formatting"
```
