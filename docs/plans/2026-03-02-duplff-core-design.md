# duplff-core Design Document

**Date:** 2026-03-02
**Status:** Approved
**Scope:** Core library crate (MVP) — no GUI/TUI

## Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Directory walker | `ignore` crate | .gitignore support, parallel walking, ripgrep ecosystem |
| Partial hash strategy | First 4KB only | Simple, one seek, catches ~90%+ non-duplicates |
| Ranking heuristic | Priority paths > deepest path > newest mtime > lexicographic | Deterministic, explainable |
| Core API style | Callback-based progress via `ProgressHandler` trait | Trivial UI integration later, `NoopProgress` for tests |

## Project Structure

```
duplff/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── duplff-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs          # Public API + orchestrator
│   │       ├── error.rs        # DuplffError enum
│   │       ├── models.rs       # FileEntry, DuplicateGroup, ScanConfig, etc.
│   │       ├── scanner.rs      # Directory walking + metadata collection
│   │       ├── hasher.rs       # Partial + full BLAKE3 hashing
│   │       ├── deduper.rs      # Size→hash grouping pipeline
│   │       ├── ranker.rs       # Keep-file selection + explanations
│   │       ├── actions.rs      # Dry-run, trash, undo
│   │       └── progress.rs     # ProgressHandler trait + NoopProgress
│   ├── duplff-cli/         # (future)
│   └── duplff-gui/         # (future)
├── docs/
└── README.md
```

## Dependencies (duplff-core)

| Crate | Purpose |
|-------|---------|
| `blake3` | BLAKE3 hashing |
| `ignore` | Parallel directory walking with .gitignore |
| `rayon` | Parallel hashing across cores |
| `trash` | OS trash deletion |
| `thiserror` | Ergonomic error types |
| `serde`, `serde_json` | Serializable models |
| `tempfile` (dev) | Test fixtures |

## Core Data Models

```rust
pub struct ScanConfig {
    pub roots: Vec<PathBuf>,
    pub extensions: Option<Vec<String>>,
    pub min_size: u64,                    // Default: 1
    pub max_size: Option<u64>,
    pub priority_paths: Vec<PathBuf>,
    pub follow_symlinks: bool,            // Default: false
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

pub struct DuplicateGroup {
    pub hash: [u8; 32],
    pub size: u64,
    pub keep: RankedFile,
    pub duplicates: Vec<RankedFile>,
}

pub struct RankedFile {
    pub entry: FileEntry,
    pub reason: KeepReason,
}

pub enum KeepReason {
    PriorityPath,
    DeepestPath,
    NewestModification,
    LexicographicFirst,
}

pub struct DuplicateReport {
    pub groups: Vec<DuplicateGroup>,
    pub total_files_scanned: usize,
    pub total_bytes_scanned: u64,
    pub total_duplicates: usize,
    pub total_wasted_bytes: u64,
}

pub enum DuplffError {
    Io(std::io::Error),
    ScanError(String),
    HashError(String),
    TrashError(String),
}
```

## Pipeline

```
Scanner ──► Deduper (size groups) ──► Partial Hasher ──► Full Hasher ──► Ranker
  │               │                       │                  │              │
[FileEntry]  [Vec<Vec<FileEntry>>]  [Vec<Vec<FileEntry>>]  [groups]   [DuplicateGroup]
```

### Stage 1 — Scanner (`scanner.rs`)
`ignore::WalkBuilder` with parallel walking. Collects `FileEntry` structs. Applies extension and min_size filters during walking. Calls `progress.on_scan_progress()` periodically.

### Stage 2 — Deduper (`deduper.rs`)
Groups `FileEntry` by size. Drops unique-size entries (can't be duplicates).

### Stage 3 — Hasher (`hasher.rs`)
- `partial_hash(path, 4096)` — first 4KB through BLAKE3
- `full_hash(path, 128KB buffer)` — stream full file through BLAKE3
- Uses `rayon::par_iter` for parallelism
- After partial hash: drop unique-hash files. Remaining get full-hashed.

### Stage 4 — Ranker (`ranker.rs`)
Scores each file in a group by (in order):
1. Priority path match (highest)
2. Path depth (deeper = more specific = better)
3. Newest mtime
4. Lexicographic path (tiebreaker)

Highest score = keep. Each file gets a `KeepReason`.

### Stage 5 — Actions (`actions.rs`)
- `dry_run(groups) -> ActionPlan`
- `trash_duplicates(groups) -> Result<ActionLog>`
- `undo(log) -> Result<()>`

## Orchestrator API

```rust
pub fn find_duplicates(
    config: &ScanConfig,
    progress: &dyn ProgressHandler,
) -> Result<DuplicateReport, DuplffError>;
```

## Testing Strategy

**Unit tests per module:**
- `scanner` — scan temp dirs, verify counts and metadata
- `hasher` — hash known content, verify BLAKE3 output
- `deduper` — same/different size files, verify grouping
- `ranker` — all criteria with controlled inputs, verify determinism
- `actions` — dry-run correctness, trash moves files

**Integration test:**
- Temp dir with known duplicates, same-size-different-content, unique files
- Full pipeline, assert correct groups/keep/counts
