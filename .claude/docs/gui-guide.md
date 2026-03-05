# duplff-gui Internals

## Stack

- **Backend**: Tauri v2 (Rust) at `crates/duplff-gui/src-tauri/`
- **Frontend**: SvelteKit (Svelte 5) at `crates/duplff-gui/src/`
- **Styling**: Tailwind CSS v4 (CSS-based config, `@import "tailwindcss"` + `@theme`)
- **Adapter**: `@sveltejs/adapter-static` (SPA mode, `fallback: "index.html"`)

## Tauri Backend

### State (`state.rs`)

```rust
pub struct AppState {
    pub report: Arc<Mutex<Option<DuplicateReport>>>,
    pub last_action_log: Arc<Mutex<Option<ActionLog>>>,
    pub scan_running: Arc<AtomicBool>,
}
```

`TauriProgress` implements `ProgressHandler` and emits events via `AppHandle`:
- `scan-progress` (every 500 files) — payload: `usize`
- `hash-progress` (every 50 files) — payload: `{ done, total }`
- `scan-complete` — payload: `DuplicateReport`
- `scan-error` — payload: `String`

### Commands (`commands.rs`)

| Command | Signature | Notes |
|---------|-----------|-------|
| `start_scan` | `(config: ScanConfig) -> ()` | Spawns thread, emits events |
| `get_results` | `() -> Option<DuplicateReport>` | Returns cached report |
| `trash_files` | `(paths: Vec<PathBuf>) -> TrashResult` | Uses `trash::delete`, saves ActionLog, updates report in-place |
| `undo_last` | `() -> UndoResult` | Undoes last action log |
| `export_json` | `() -> String` | Pretty JSON of report |
| `export_csv` | `() -> String` | CSV: Group, Hash, Path, Size, Status, Reason |
| `open_in_file_manager` | `(path: PathBuf) -> ()` | `open::that()` on parent dir |
| `dry_run` | `() -> DryRunPlan` | Preview deletion plan |
| `list_action_logs` | `() -> Vec<ActionLogSummary>` | All past trash operations |
| `undo_log` | `(timestamp: String) -> UndoResult` | Undo specific past operation |

### Registration (`lib.rs`)

All commands registered in `tauri::generate_handler![]`. Plugins: `tauri_plugin_opener`, `tauri_plugin_dialog`.

### Capabilities (`capabilities/default.json`)

Permissions: `core:default`, `opener:default`, `dialog:default`

## Svelte Frontend

### Screen Flow

```
Setup → Progress → Results ⇄ Detail
  ↑                   |
  └───── New Scan ────┘
```

Screen transitions controlled by the `currentScreen` store.

### Stores (`stores.ts`)

| Store | Type | Purpose |
|-------|------|---------|
| `currentScreen` | `Screen` | Active screen ('setup' / 'progress' / 'results' / 'detail') |
| `scanConfig` | `ScanConfig` | Current scan configuration |
| `report` | `DuplicateReport \| null` | Scan results |
| `selectedGroup` | `number` | Index of selected group for detail view |
| `markedFiles` | `Set<string>` | File paths marked for deletion |
| `filterText` | `string` | Search filter in results |
| `sortMode` | `SortMode` | Sort order ('wasted' / 'size' / 'files' / 'path') |

### API Layer (`api.ts`)

Thin wrappers around `invoke()` and `listen()` from `@tauri-apps/api`. Every Tauri command has a corresponding async function. Event listeners return `UnlistenFn` for cleanup in `onDestroy`.

### Types (`types.ts`)

TypeScript interfaces that mirror Rust types exactly. Field names use snake_case to match Tauri serialization. Includes `defaultConfig()` factory.

### Screens

**Setup.svelte**: Form with FolderPicker for roots, size inputs with unit selectors, extensions input, collapsible Advanced section (exclude patterns, priority dirs, paranoid, no-cache, symlinks). Scan button calls `startScan()` and transitions to progress.

**Progress.svelte**: Centered card. Listens to scan-progress/hash-progress/scan-complete/scan-error events. Thin animated progress bar. Shows files found, hash progress, elapsed time. Cancel returns to setup.

**Results.svelte**: Header with stats (groups, duplicates, wasted). Toolbar with filter input and sort dropdown. GroupTable component. Action bar with select/clear/trash/undo/export. Toast notifications. ConfirmDialog for trash.

**Detail.svelte**: Breadcrumb header (Results > group stats). FileList component. Same action bar pattern. Per-group selection tracking.

### Components

**FolderPicker**: Tauri dialog integration. Displays selected folders as chips with remove button. "+ Add folder" dashed-border button.

**GroupTable**: Table with columns: Files, Size, Wasted (with proportional bar), Path. Filters by `filterText`, sorts by `sortMode`. Click navigates to detail.

**FileList**: Keep file with green left border. Duplicates with checkboxes. Open-in-file-manager button on hover. Reason labels as subtle text.

**ConfirmDialog**: Frosted glass backdrop (`backdrop-blur-sm`). Card with title, message, cancel/confirm buttons.

### Styling Conventions

- Tailwind utility classes exclusively (no custom CSS beyond `app.css`)
- Dark theme: `bg-gray-900` base, `gray-800` for cards/surfaces
- Custom theme colors in `app.css` `@theme` block: keep, delete, warn, info, active
- Typography: `text-xs` for most UI, `font-mono` for numbers/paths
- Transitions: `transition-colors` on interactive elements
- Custom scrollbar styling in `app.css`

### Config Files

- `svelte.config.js` — adapter-static, SPA fallback
- `vite.config.ts` — port 1420, Tailwind + SvelteKit plugins
- `tauri.conf.json` — window 1000x700, min 800x500, bundle targets: all
