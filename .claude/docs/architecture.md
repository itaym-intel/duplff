# Architecture

## Crate Dependency Graph

```
duplff-cli ──→ duplff-core
duplff-gui ──→ duplff-core
```

Both CLI and GUI depend on core. They never depend on each other.

## duplff-core Modules

| Module | Responsibility |
|--------|---------------|
| `models.rs` | All data types: ScanConfig, FileEntry, HashedFile, DuplicateGroup, RankedFile, KeepReason, DuplicateReport |
| `error.rs` | DuplffError enum (Io, ScanError, HashError, TrashError, CacheError, LogError), Result alias |
| `scanner.rs` | Directory walking with `ignore` crate. Filters by extension, size, exclude patterns. Parallel via `build_parallel()`. |
| `hasher.rs` | `partial_hash()` (first 4KB) and `full_hash()` (128KB buffer) using BLAKE3 |
| `deduper.rs` | `group_by_size()` then `find_duplicate_groups()` — cascading hash refinement with optional cache and paranoid verification |
| `ranker.rs` | `rank_group()` / `rank_groups()` — picks keep file per group using priority ordering |
| `actions.rs` | `dry_run()`, `trash_duplicates()`, `undo()` — file operations with ActionLog/ActionPlan types |
| `cache.rs` | SQLite-backed HashCache at ~/.cache/duplff/hashes.db. Keyed by (path, size, mtime). |
| `log_store.rs` | Save/load/list action logs as JSON at ~/.local/share/duplff/logs/ |
| `verify.rs` | `files_identical()` and `verify_group()` for byte-by-byte paranoid mode |
| `progress.rs` | `ProgressHandler` trait with on_scan_progress, on_hash_progress, on_complete. `NoopProgress` for tests. |
| `lib.rs` | `find_duplicates()` orchestrator — the single public entry point |

## duplff-cli Structure

| File | Purpose |
|------|---------|
| `main.rs` | Entry point — parses args, dispatches to TUI or non-interactive |
| `cli.rs` | Clap argument definitions |
| `format.rs` | `human_bytes()`, `truncate_path()` utilities |
| `non_interactive.rs` | `run_json()`, `run_dry_run()`, `run_csv()` |
| `tui/mod.rs` | Main TUI loop — rendering, input handling |
| `tui/state.rs` | AppState enum (Scanning, Results, Confirm, Help, Error), FocusPane, SortMode |
| `tui/scan.rs` | Background scanner thread with ChannelProgress |
| `tui/groups.rs` | Groups pane rendering |
| `tui/detail.rs` | Detail pane rendering |
| `tui/help.rs` | Help overlay |

## duplff-gui Structure

### Rust Backend (src-tauri/)

| File | Purpose |
|------|---------|
| `lib.rs` | Tauri app builder — registers plugins, state, and 10 commands |
| `commands.rs` | All Tauri commands (start_scan, get_results, trash_files, undo_last, export_json, export_csv, open_in_file_manager, dry_run, list_action_logs, undo_log) |
| `state.rs` | AppState (Arc<Mutex<...>> fields), TauriProgress (ProgressHandler impl that emits Tauri events) |

### Svelte Frontend (src/)

| File | Purpose |
|------|---------|
| `routes/+page.svelte` | Main page — renders current screen based on store |
| `lib/types.ts` | TypeScript interfaces mirroring Rust types |
| `lib/api.ts` | Tauri invoke wrappers + event listeners |
| `lib/stores.ts` | Svelte writable stores (currentScreen, scanConfig, report, selectedGroup, markedFiles, filterText, sortMode) |
| `lib/format.ts` | formatBytes, truncatePath, reasonLabel |
| `lib/screens/Setup.svelte` | Scan config form with collapsible Advanced section |
| `lib/screens/Progress.svelte` | Real-time scan/hash progress |
| `lib/screens/Results.svelte` | Groups table with sort/filter, trash, export |
| `lib/screens/Detail.svelte` | Single group detail with file list |
| `lib/components/FolderPicker.svelte` | Tauri dialog folder selector with chips |
| `lib/components/GroupTable.svelte` | Sortable/filterable groups table |
| `lib/components/FileList.svelte` | Keep + duplicate file list with checkboxes |
| `lib/components/ConfirmDialog.svelte` | Modal confirmation with frosted backdrop |

## IPC Pattern (GUI)

```
Frontend (Svelte) ──invoke()──→ Tauri Command (Rust) ──→ duplff-core
                   ←─result──┘

Rust ──emit()──→ Frontend (listen())   [for async progress events]
```

Events emitted: `scan-progress`, `hash-progress`, `scan-complete`, `scan-error`

## State Management (GUI)

Svelte writable stores in `stores.ts` are the single source of truth. Screens read from stores and write back after Tauri command responses. No prop drilling beyond one level.

## Parallelism

- **Directory walking**: `ignore` crate's `build_parallel()` (thread pool)
- **Hashing**: `rayon` parallel iterators
- **GUI scan**: spawned `std::thread` with `TauriProgress` emitting events back to the frontend
