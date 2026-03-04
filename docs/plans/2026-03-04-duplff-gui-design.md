# duplff GUI Design

## Overview

A native Linux desktop GUI for duplff built with Tauri v2 + Svelte 5 + Tailwind CSS. The GUI provides an accessible interface for non-CLI users to find and remove duplicate files. It reuses `duplff-core` for all scanning, hashing, ranking, and file operations.

## Tech Stack

- **Tauri v2** — Rust backend + system WebView. Produces a native binary (<10MB).
- **Svelte 5** — Frontend framework. Compiles to vanilla JS, minimal bundle size.
- **Tailwind CSS v4** — Utility-first styling.
- **duplff-core** — Shared core library (scanning, hashing, deduplication, actions).

## Crate Structure

```
crates/duplff-gui/
├── Cargo.toml              # Tauri Rust backend
├── tauri.conf.json         # Tauri config (window size, title, permissions)
├── src-tauri/
│   ├── src/
│   │   ├── main.rs         # Tauri app entry point
│   │   ├── commands.rs     # Tauri command handlers (invoke from JS)
│   │   └── state.rs        # App state (scan results, progress)
│   ├── Cargo.toml
│   └── capabilities/       # Tauri v2 permissions
├── src/                    # Svelte frontend
│   ├── App.svelte          # Root component + screen routing
│   ├── lib/
│   │   ├── api.ts          # Typed wrappers around Tauri invoke/listen
│   │   ├── types.ts        # TypeScript types matching Rust models
│   │   └── stores.ts       # Svelte stores for shared state
│   ├── screens/
│   │   ├── Setup.svelte    # Scan configuration screen
│   │   ├── Progress.svelte # Scanning/hashing progress screen
│   │   ├── Results.svelte  # Duplicate groups table
│   │   └── Detail.svelte   # Group detail with file list and actions
│   └── components/
│       ├── FolderPicker.svelte
│       ├── GroupTable.svelte
│       ├── FileList.svelte
│       ├── ActionBar.svelte
│       └── ConfirmDialog.svelte
├── package.json
├── svelte.config.js
├── vite.config.ts
└── tailwind.config.js
```

## Screens

### 1. Scan Setup

The landing screen. Users configure a scan before starting.

**Elements:**
- Folder picker (+ button / drag-drop area) — adds root directories to scan
- List of selected folders with remove (x) buttons
- Extension filter — text input for comma-separated extensions (e.g. "py,rs,js")
- Min size input — numeric, defaults to 1 KB
- Max size input — optional
- Exclude patterns — text input for comma-separated patterns (e.g. "node_modules,.git")
- Priority directories — optional folder picker for "source of truth" paths
- Checkboxes: Follow symlinks, Paranoid mode, No cache
- "Scan" button (primary, prominent)

**Layout:** Single centered card, vertically stacked fields. Clean and minimal.

### 2. Progress

Shown while scan is running.

**Elements:**
- Phase indicator: "Scanning files..." or "Hashing files..."
- Progress bar (determinate during hashing, indeterminate during scanning)
- Stats: files found, files hashed / total, elapsed time
- Current file path being processed (scrolling log or single-line)
- "Cancel" button

**Data flow:** Tauri backend emits `scan-progress` and `hash-progress` events. Frontend listens and updates the UI reactively.

### 3. Results View

Main screen after scan completes.

**Elements:**
- Summary bar: "N groups, M duplicates, X wasted" (like the TUI)
- Sortable table of duplicate groups with columns: #, Files, Size, Wasted, Sample Path
- Click a row to navigate to the Detail screen for that group
- Sort controls: click column headers to sort
- Filter input: text box to filter groups by path (like TUI's `/`)
- "Auto-Select All Duplicates" button — marks all non-keep files across all groups
- "Trash Marked" button — deletes all marked files (with confirmation)
- "Export CSV" / "Export JSON" buttons for non-interactive export

**Layout:** Full-width table with a sticky summary bar at top and action bar at bottom.

### 4. Group Detail

Shown when clicking a group from the Results table.

**Elements:**
- Back button to return to Results
- Group info header: hash, file size, file count
- File list with checkboxes:
  - Keep file: green badge with reason (e.g. "Kept: deepest path")
  - Duplicate files: red/neutral with checkbox to mark for deletion
- "Select All Duplicates" / "Deselect All" buttons
- "Trash Selected" button (with confirmation dialog)
- "Open Folder" button — opens the file's parent directory in the system file manager

**Layout:** Left side is the file list, full width. Future: right side will be a preview pane (images, code diffs).

## Tauri Backend

### Commands (Rust → invoked from JS)

```rust
#[tauri::command]
async fn start_scan(config: ScanConfig, app: AppHandle) -> Result<(), String>

#[tauri::command]
fn get_results(state: State<AppState>) -> Option<DuplicateReport>

#[tauri::command]
fn trash_files(paths: Vec<PathBuf>, state: State<AppState>) -> Result<TrashResult, String>

#[tauri::command]
fn undo_last(state: State<AppState>) -> Result<UndoResult, String>

#[tauri::command]
fn pick_folder() -> Result<Option<PathBuf>, String>

#[tauri::command]
fn open_in_file_manager(path: PathBuf) -> Result<(), String>

#[tauri::command]
fn export_csv(state: State<AppState>) -> Result<String, String>

#[tauri::command]
fn export_json(state: State<AppState>) -> Result<String, String>
```

### Events (Rust → pushed to JS)

```
"scan-progress"  → { files_found: usize }
"hash-progress"  → { done: usize, total: usize }
"scan-complete"  → { report: DuplicateReport }
"scan-error"     → { message: String }
```

### State Management

```rust
struct AppState {
    report: Mutex<Option<DuplicateReport>>,
    last_action_log: Mutex<Option<ActionLog>>,
}
```

The `start_scan` command spawns a background thread (like the TUI does), uses a `ProgressHandler` that emits Tauri events instead of sending through mpsc channels.

### Serialization

`DuplicateReport` and related types already derive `Serialize`/`Deserialize` (serde), so they pass through Tauri's command/event system automatically.

## Frontend State

Svelte stores manage screen routing and shared data:

```typescript
// stores.ts
export const currentScreen = writable<'setup' | 'progress' | 'results' | 'detail'>('setup');
export const scanConfig = writable<ScanConfig>(defaultConfig());
export const report = writable<DuplicateReport | null>(null);
export const selectedGroup = writable<number>(0);
export const markedFiles = writable<Set<string>>(new Set());
export const filterText = writable<string>('');
export const sortMode = writable<SortMode>('wasted');
```

## Data Flow

```
User clicks "Scan"
  → JS calls invoke('start_scan', { config })
  → Rust spawns background thread
  → Thread calls duplff_core::find_duplicates() with TauriProgress handler
  → TauriProgress emits events: scan-progress, hash-progress
  → JS listens, updates Progress screen reactively
  → On complete: Rust emits scan-complete with full report
  → JS stores report, switches to Results screen
  → User marks files, clicks "Trash Marked"
  → JS calls invoke('trash_files', { paths })
  → Rust calls duplff_core::actions::trash_duplicates()
  → Rust saves action log via log_store
  → Returns result to JS, JS updates report
```

## Styling

Tailwind CSS v4 with a dark theme by default (matches typical developer tooling). Color conventions:
- Green: keep/safe actions
- Red: delete/destructive actions
- Yellow: warnings
- Gray: neutral/informational
- Cyan: active/focused elements

## MVP Scope (this plan)

**Included:**
- All 4 screens (Setup, Progress, Results, Detail)
- Folder picker via Tauri dialog API
- Sort and filter on Results
- Mark/unmark files, trash with confirmation
- Undo last trash operation
- Auto-select all duplicates
- Export CSV/JSON
- Action log persistence

**Deferred to post-MVP:**
- Image thumbnail previews
- Code/text diff viewer
- Drag-and-drop folder input
- Open file in default app
- Keyboard shortcuts
- Multiple themes / light mode
