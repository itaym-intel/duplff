# duplff-core API Reference

## Entry Point

```rust
pub fn find_duplicates(config: &ScanConfig, progress: &dyn ProgressHandler) -> Result<DuplicateReport>
```

## Models (`models.rs`)

```rust
pub struct ScanConfig {
    pub roots: Vec<PathBuf>,
    pub extensions: Option<Vec<String>>,
    pub min_size: u64,              // default: 1
    pub max_size: Option<u64>,
    pub priority_paths: Vec<PathBuf>,
    pub follow_symlinks: bool,      // default: false
    pub exclude_patterns: Vec<String>,
    pub no_cache: bool,             // default: false
    pub paranoid: bool,             // default: false
}

pub struct FileEntry {
    pub path: PathBuf,
    pub size: u64,
    pub modified: SystemTime,
}

pub struct HashedFile {
    pub entry: FileEntry,
    pub hash: [u8; 32],
}

pub enum KeepReason {
    PriorityPath,
    DeepestPath,
    NewestModification,
    LexicographicFirst,
}
// Implements Display: "priority path", "deepest path", etc.

pub struct RankedFile {
    pub entry: FileEntry,
    pub reason: KeepReason,
}

pub struct DuplicateGroup {
    pub hash: [u8; 32],
    pub size: u64,
    pub keep: RankedFile,
    pub duplicates: Vec<RankedFile>,
}
// Method: wasted_bytes() -> u64

pub struct DuplicateReport {
    pub groups: Vec<DuplicateGroup>,
    pub total_files_scanned: usize,
    pub total_bytes_scanned: u64,
    pub total_duplicates: usize,
    pub total_wasted_bytes: u64,
}
```

All model types derive `Serialize` and `Deserialize`.

## Actions (`actions.rs`)

```rust
pub struct ActionPlan { pub files_to_delete: Vec<PathBuf>, pub bytes_to_reclaim: u64 }
pub struct ActionRecord { pub path: PathBuf, pub action: ActionType }
pub enum ActionType { Trashed }
pub struct ActionLog { pub actions: Vec<ActionRecord>, pub bytes_reclaimed: u64, pub timestamp: String }
pub struct UndoResult { pub restored: Vec<PathBuf>, pub not_found: Vec<PathBuf> }

pub fn dry_run(groups: &[DuplicateGroup]) -> ActionPlan
pub fn trash_duplicates(groups: &[DuplicateGroup]) -> Result<ActionLog>
pub fn undo(log: &ActionLog) -> Result<UndoResult>
```

## Scanner (`scanner.rs`)

```rust
pub fn scan(config: &ScanConfig, progress: &dyn ProgressHandler) -> Result<Vec<FileEntry>>
```

## Hasher (`hasher.rs`)

```rust
pub fn partial_hash(path: &Path) -> Result<[u8; 32]>  // first 4KB
pub fn full_hash(path: &Path) -> Result<[u8; 32]>     // 128KB buffer
```

## Deduper (`deduper.rs`)

```rust
pub fn group_by_size(files: Vec<FileEntry>) -> Vec<Vec<FileEntry>>
pub fn find_duplicate_groups(
    files: Vec<FileEntry>,
    progress: &dyn ProgressHandler,
    cache: Option<&HashCache>,
    paranoid: bool,
) -> Result<Vec<Vec<HashedFile>>>
```

## Ranker (`ranker.rs`)

```rust
pub fn rank_group(files: Vec<HashedFile>, priority_paths: &[PathBuf]) -> DuplicateGroup
pub fn rank_groups(groups: Vec<Vec<HashedFile>>, priority_paths: &[PathBuf]) -> Vec<DuplicateGroup>
```

## Cache (`cache.rs`)

```rust
pub struct HashCache { /* SQLite connection */ }
impl HashCache {
    pub fn open_default() -> Result<Self>              // ~/.cache/duplff/hashes.db
    pub fn open(path: &Path) -> Result<Self>
    pub fn get_partial(&self, path: &Path, size: u64, mtime: u64) -> Option<[u8; 32]>
    pub fn get_full(&self, path: &Path, size: u64, mtime: u64) -> Option<[u8; 32]>
    pub fn put_partial(&self, path: &Path, size: u64, mtime: u64, hash: &[u8; 32])
    pub fn put_full(&self, path: &Path, size: u64, mtime: u64, hash: &[u8; 32])
}
```

## Log Store (`log_store.rs`)

```rust
pub fn save_action_log(log: &ActionLog) -> Result<PathBuf>
pub fn load_latest_log() -> Result<ActionLog>
pub fn load_log(path: &Path) -> Result<ActionLog>
pub fn list_logs() -> Result<Vec<PathBuf>>
```

Logs stored at `~/.local/share/duplff/logs/{timestamp}.json`.

## Verify (`verify.rs`)

```rust
pub fn files_identical(path_a: &Path, path_b: &Path) -> Result<bool>
pub fn verify_group(paths: &[&Path]) -> Result<bool>
```

## Progress (`progress.rs`)

```rust
pub trait ProgressHandler: Send + Sync {
    fn on_scan_progress(&self, files_found: usize);
    fn on_hash_progress(&self, files_hashed: usize, total: usize);
    fn on_complete(&self, groups_found: usize);
}

pub struct NoopProgress;  // implements ProgressHandler as no-ops
```

## Error (`error.rs`)

```rust
pub enum DuplffError {
    Io(std::io::Error),
    ScanError(String),
    HashError(String),
    TrashError(String),
    CacheError(String),
    LogError(String),
}
pub type Result<T> = std::result::Result<T, DuplffError>;
```
