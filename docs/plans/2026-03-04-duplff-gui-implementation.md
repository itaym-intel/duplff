# duplff GUI Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a native Linux desktop GUI for duplff using Tauri v2 + Svelte 5 + Tailwind CSS v4, reusing duplff-core for all scanning, hashing, ranking, and file operations.

**Architecture:** A Tauri v2 app with a Rust backend that wraps duplff-core, communicating with a Svelte 5 frontend via Tauri commands (JS→Rust) and events (Rust→JS). The frontend has 4 screens: Setup, Progress, Results, Detail.

**Tech Stack:** Rust (Tauri v2), Svelte 5, Tailwind CSS v4, TypeScript, Vite, duplff-core

**Design doc:** `docs/plans/2026-03-04-duplff-gui-design.md`

---

### Task 1: Scaffold Tauri + Svelte Project

**Files:**
- Create: `crates/duplff-gui/` (entire scaffolded project)
- Modify: `Cargo.toml` (workspace root — add member)

**Step 1: Install system dependencies**

Run:
```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

**Step 2: Scaffold the Tauri project**

```bash
cd /home/itay/GitHub/duplff
npx create-tauri-app@latest crates/duplff-gui --template svelte-ts --manager npm
```

If the CLI is interactive, choose: Svelte, TypeScript, npm.

**Step 3: Install frontend dependencies**

```bash
cd crates/duplff-gui
npm install
```

**Step 4: Install Tailwind CSS v4**

```bash
cd crates/duplff-gui
npm install tailwindcss @tailwindcss/vite
```

**Step 5: Configure Tailwind in Vite**

Modify `crates/duplff-gui/vite.config.ts`:
```typescript
import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  plugins: [tailwindcss(), svelte()],
});
```

**Step 6: Add Tailwind CSS import**

Create/modify `crates/duplff-gui/src/app.css`:
```css
@import "tailwindcss";

@theme {
  --color-keep: oklch(0.723 0.191 142.5);
  --color-delete: oklch(0.637 0.237 25.3);
  --color-warn: oklch(0.795 0.184 86.0);
  --color-info: oklch(0.551 0.014 264.5);
  --color-active: oklch(0.715 0.143 215.2);
}
```

**Step 7: Add workspace member**

Modify `Cargo.toml` (root):
```toml
[workspace]
members = ["crates/duplff-core", "crates/duplff-cli", "crates/duplff-gui/src-tauri"]
resolver = "2"
```

**Step 8: Add duplff-core dependency to Tauri backend**

Modify `crates/duplff-gui/src-tauri/Cargo.toml` — add to `[dependencies]`:
```toml
duplff-core = { path = "../../duplff-core" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
csv = "1"
hex = "0.4"
open = "5"
```

**Step 9: Configure tauri.conf.json**

Modify `crates/duplff-gui/src-tauri/tauri.conf.json`:
- Set `productName` to `"duplff"`
- Set `identifier` to `"com.duplff.app"`
- Set window `title` to `"duplff — Duplicate File Finder"`
- Set window `width` to 1000, `height` to 700
- Set window `minWidth` to 800, `minHeight` to 500

**Step 10: Install dialog plugin**

```bash
cd crates/duplff-gui
npm install @tauri-apps/plugin-dialog
```

Add to `src-tauri/Cargo.toml`:
```toml
tauri-plugin-dialog = "2"
```

Register in `src-tauri/src/main.rs`:
```rust
tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
```

**Step 11: Verify build**

```bash
cd crates/duplff-gui && npm run tauri dev
```

Expected: A window opens with the Svelte template content.

**Step 12: Commit**

```bash
git add crates/duplff-gui Cargo.toml
git commit -m "feat: scaffold duplff-gui Tauri + Svelte project"
```

---

### Task 2: Implement Tauri Backend — State, Progress, and Commands

**Files:**
- Create: `crates/duplff-gui/src-tauri/src/state.rs`
- Create: `crates/duplff-gui/src-tauri/src/commands.rs`
- Modify: `crates/duplff-gui/src-tauri/src/main.rs`

**Step 1: Write state.rs**

Create `crates/duplff-gui/src-tauri/src/state.rs`:
```rust
use duplff_core::actions::ActionLog;
use duplff_core::models::DuplicateReport;
use duplff_core::progress::ProgressHandler;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

/// Shared application state managed by Tauri.
pub struct AppState {
    pub report: Arc<Mutex<Option<DuplicateReport>>>,
    pub last_action_log: Arc<Mutex<Option<ActionLog>>>,
    pub scan_running: Arc<AtomicBool>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            report: Arc::new(Mutex::new(None)),
            last_action_log: Arc::new(Mutex::new(None)),
            scan_running: Arc::new(AtomicBool::new(false)),
        }
    }
}

/// Progress handler that emits Tauri events to the frontend.
pub struct TauriProgress {
    app: AppHandle,
    scan_count: AtomicUsize,
    hash_count: AtomicUsize,
}

impl TauriProgress {
    pub fn new(app: AppHandle) -> Self {
        Self {
            app,
            scan_count: AtomicUsize::new(0),
            hash_count: AtomicUsize::new(0),
        }
    }
}

impl ProgressHandler for TauriProgress {
    fn on_scan_progress(&self, files_found: usize) {
        let prev = self.scan_count.swap(files_found, Ordering::Relaxed);
        if files_found / 500 != prev / 500 || files_found == 0 {
            let _ = self.app.emit("scan-progress", files_found);
        }
    }

    fn on_hash_progress(&self, files_hashed: usize, total: usize) {
        let prev = self.hash_count.swap(files_hashed, Ordering::Relaxed);
        if files_hashed / 50 != prev / 50 {
            let _ = self.app.emit("hash-progress", serde_json::json!({
                "done": files_hashed,
                "total": total,
            }));
        }
    }

    fn on_complete(&self, _groups_found: usize) {
        // Complete event is emitted after find_duplicates returns
    }
}
```

**Step 2: Write commands.rs**

Create `crates/duplff-gui/src-tauri/src/commands.rs`:
```rust
use crate::state::{AppState, TauriProgress};
use duplff_core::models::ScanConfig;
use serde::Serialize;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, State};

#[derive(Serialize)]
pub struct TrashResult {
    pub count: usize,
    pub bytes_reclaimed: u64,
}

#[derive(Serialize)]
pub struct UndoResult {
    pub restored: usize,
    pub not_found: usize,
}

#[tauri::command]
pub async fn start_scan(
    config: ScanConfig,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if state.scan_running.load(Ordering::Relaxed) {
        return Err("scan already running".into());
    }
    state.scan_running.store(true, Ordering::Relaxed);
    *state.report.lock().unwrap() = None;

    let report_store = state.report.clone();
    let scan_flag = state.scan_running.clone();

    std::thread::spawn(move || {
        let progress = TauriProgress::new(app.clone());
        match duplff_core::find_duplicates(&config, &progress) {
            Ok(report) => {
                *report_store.lock().unwrap() = Some(report.clone());
                let _ = app.emit("scan-complete", &report);
            }
            Err(e) => {
                let _ = app.emit("scan-error", e.to_string());
            }
        }
        scan_flag.store(false, Ordering::Relaxed);
    });

    Ok(())
}

#[tauri::command]
pub fn get_results(state: State<'_, AppState>) -> Option<duplff_core::models::DuplicateReport> {
    state.report.lock().unwrap().clone()
}

#[tauri::command]
pub fn trash_files(
    paths: Vec<std::path::PathBuf>,
    state: State<'_, AppState>,
) -> Result<TrashResult, String> {
    let mut trashed = Vec::new();
    let mut bytes_reclaimed = 0u64;

    for path in &paths {
        if let Ok(meta) = std::fs::metadata(path) {
            bytes_reclaimed += meta.len();
        }
        trash::delete(path).map_err(|e| format!("{}: {e}", path.display()))?;
        trashed.push(path.clone());
    }

    let log = duplff_core::actions::ActionLog {
        actions: trashed
            .iter()
            .map(|p| duplff_core::actions::ActionRecord {
                path: p.clone(),
                action: duplff_core::actions::ActionType::Trashed,
            })
            .collect(),
        bytes_reclaimed,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .to_string(),
    };

    let _ = duplff_core::log_store::save_action_log(&log);
    *state.last_action_log.lock().unwrap() = Some(log);

    // Remove trashed files from the stored report
    if let Some(ref mut report) = *state.report.lock().unwrap() {
        for group in &mut report.groups {
            group.duplicates.retain(|d| !trashed.contains(&d.entry.path));
        }
        report.groups.retain(|g| !g.duplicates.is_empty());
        report.total_duplicates = report.groups.iter().map(|g| g.duplicates.len()).sum();
        report.total_wasted_bytes = report.groups.iter().map(|g| g.wasted_bytes()).sum();
    }

    Ok(TrashResult {
        count: trashed.len(),
        bytes_reclaimed,
    })
}

#[tauri::command]
pub fn undo_last(state: State<'_, AppState>) -> Result<UndoResult, String> {
    let log = state.last_action_log.lock().unwrap();
    let log = log.as_ref().ok_or("no action to undo")?;
    let result = duplff_core::actions::undo(log).map_err(|e| e.to_string())?;
    Ok(UndoResult {
        restored: result.restored.len(),
        not_found: result.not_found.len(),
    })
}

#[tauri::command]
pub fn export_json(state: State<'_, AppState>) -> Result<String, String> {
    let report = state.report.lock().unwrap();
    let report = report.as_ref().ok_or("no results to export")?;
    serde_json::to_string_pretty(report).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_csv(state: State<'_, AppState>) -> Result<String, String> {
    let report = state.report.lock().unwrap();
    let report = report.as_ref().ok_or("no results to export")?;
    let mut wtr = csv::Writer::from_writer(vec![]);
    wtr.write_record(["Group", "Status", "Path", "Size", "Hash"])
        .map_err(|e| e.to_string())?;
    for (i, group) in report.groups.iter().enumerate() {
        let hash = hex::encode(group.hash);
        wtr.write_record([
            &(i + 1).to_string(),
            "keep",
            &group.keep.entry.path.display().to_string(),
            &group.size.to_string(),
            &hash,
        ])
        .map_err(|e| e.to_string())?;
        for dup in &group.duplicates {
            wtr.write_record([
                &(i + 1).to_string(),
                "duplicate",
                &dup.entry.path.display().to_string(),
                &group.size.to_string(),
                &hash,
            ])
            .map_err(|e| e.to_string())?;
        }
    }
    String::from_utf8(wtr.into_inner().map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_in_file_manager(path: std::path::PathBuf) -> Result<(), String> {
    let dir = if path.is_file() {
        path.parent().unwrap_or(&path).to_path_buf()
    } else {
        path
    };
    open::that(&dir).map_err(|e| e.to_string())
}
```

**Step 3: Update main.rs**

```rust
mod commands;
mod state;

use state::AppState;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::start_scan,
            commands::get_results,
            commands::trash_files,
            commands::undo_last,
            commands::export_json,
            commands::export_csv,
            commands::open_in_file_manager,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Step 4: Set up Tauri capabilities**

Create/modify `crates/duplff-gui/src-tauri/capabilities/default.json`:
```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Default capabilities",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "dialog:default",
    "dialog:allow-open"
  ]
}
```

**Step 5: Verify it compiles**

```bash
cd crates/duplff-gui && npm run tauri dev
```

**Step 6: Commit**

```bash
git add crates/duplff-gui/src-tauri/
git commit -m "feat: add Tauri backend with all commands and state"
```

---

### Task 3: Frontend Foundation — Types, API, Stores, Routing

**Files:**
- Create: `crates/duplff-gui/src/lib/types.ts`
- Create: `crates/duplff-gui/src/lib/api.ts`
- Create: `crates/duplff-gui/src/lib/stores.ts`
- Create: `crates/duplff-gui/src/lib/format.ts`
- Modify: `crates/duplff-gui/src/App.svelte`
- Modify: `crates/duplff-gui/src/app.css`

**Step 1: Install Tauri JS packages**

```bash
cd crates/duplff-gui
npm install @tauri-apps/api @tauri-apps/plugin-dialog
```

**Step 2: Create types.ts**

Create `crates/duplff-gui/src/lib/types.ts`:
```typescript
export interface ScanConfig {
  roots: string[];
  extensions: string[] | null;
  min_size: number;
  max_size: number | null;
  priority_paths: string[];
  follow_symlinks: boolean;
  exclude_patterns: string[];
  no_cache: boolean;
  paranoid: boolean;
}

export interface FileEntry {
  path: string;
  size: number;
  modified: number;
}

export interface RankedFile {
  entry: FileEntry;
  reason: 'PriorityPath' | 'DeepestPath' | 'NewestModification' | 'LexicographicFirst';
}

export interface DuplicateGroup {
  hash: string;
  size: number;
  keep: RankedFile;
  duplicates: RankedFile[];
}

export interface DuplicateReport {
  groups: DuplicateGroup[];
  total_files_scanned: number;
  total_bytes_scanned: number;
  total_duplicates: number;
  total_wasted_bytes: number;
}

export interface TrashResult {
  count: number;
  bytes_reclaimed: number;
}

export interface UndoResult {
  restored: number;
  not_found: number;
}

export interface HashProgress {
  done: number;
  total: number;
}

export type Screen = 'setup' | 'progress' | 'results' | 'detail';
export type SortMode = 'wasted' | 'size' | 'files' | 'path';

export function defaultConfig(): ScanConfig {
  return {
    roots: [],
    extensions: null,
    min_size: 1024,
    max_size: null,
    priority_paths: [],
    follow_symlinks: false,
    exclude_patterns: [],
    no_cache: false,
    paranoid: false,
  };
}
```

**Step 3: Create api.ts**

Create `crates/duplff-gui/src/lib/api.ts`:
```typescript
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
  ScanConfig, DuplicateReport, TrashResult, UndoResult, HashProgress
} from './types';

export async function startScan(config: ScanConfig): Promise<void> {
  return invoke('start_scan', { config });
}

export async function getResults(): Promise<DuplicateReport | null> {
  return invoke('get_results');
}

export async function trashFiles(paths: string[]): Promise<TrashResult> {
  return invoke('trash_files', { paths });
}

export async function undoLast(): Promise<UndoResult> {
  return invoke('undo_last');
}

export async function exportJson(): Promise<string> {
  return invoke('export_json');
}

export async function exportCsv(): Promise<string> {
  return invoke('export_csv');
}

export async function openInFileManager(path: string): Promise<void> {
  return invoke('open_in_file_manager', { path });
}

export function onScanProgress(cb: (filesFound: number) => void): Promise<UnlistenFn> {
  return listen<number>('scan-progress', (e) => cb(e.payload));
}

export function onHashProgress(cb: (progress: HashProgress) => void): Promise<UnlistenFn> {
  return listen<HashProgress>('hash-progress', (e) => cb(e.payload));
}

export function onScanComplete(cb: (report: DuplicateReport) => void): Promise<UnlistenFn> {
  return listen<DuplicateReport>('scan-complete', (e) => cb(e.payload));
}

export function onScanError(cb: (message: string) => void): Promise<UnlistenFn> {
  return listen<string>('scan-error', (e) => cb(e.payload));
}
```

**Step 4: Create stores.ts**

Create `crates/duplff-gui/src/lib/stores.ts`:
```typescript
import { writable } from 'svelte/store';
import type { Screen, ScanConfig, DuplicateReport, SortMode } from './types';
import { defaultConfig } from './types';

export const currentScreen = writable<Screen>('setup');
export const scanConfig = writable<ScanConfig>(defaultConfig());
export const report = writable<DuplicateReport | null>(null);
export const selectedGroup = writable<number>(0);
export const markedFiles = writable<Set<string>>(new Set());
export const filterText = writable<string>('');
export const sortMode = writable<SortMode>('wasted');
export const errorMessage = writable<string | null>(null);
```

**Step 5: Create format.ts**

Create `crates/duplff-gui/src/lib/format.ts`:
```typescript
export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return (bytes / Math.pow(1024, i)).toFixed(i > 0 ? 1 : 0) + ' ' + units[i];
}

export function truncatePath(path: string, maxLen = 60): string {
  if (path.length <= maxLen) return path;
  return '...' + path.slice(-(maxLen - 3));
}

export function reasonLabel(reason: string): string {
  const labels: Record<string, string> = {
    PriorityPath: 'Priority directory',
    DeepestPath: 'Deepest path',
    NewestModification: 'Newest file',
    LexicographicFirst: 'First alphabetically',
  };
  return labels[reason] || reason;
}
```

**Step 6: Update app.css with Tailwind dark theme**

Replace `crates/duplff-gui/src/app.css`:
```css
@import "tailwindcss";

@theme {
  --color-keep: oklch(0.723 0.191 142.5);
  --color-delete: oklch(0.637 0.237 25.3);
  --color-warn: oklch(0.795 0.184 86.0);
  --color-info: oklch(0.551 0.014 264.5);
  --color-active: oklch(0.715 0.143 215.2);
}

body {
  @apply bg-gray-900 text-gray-100 font-sans;
  margin: 0;
  padding: 0;
}
```

**Step 7: Update App.svelte with screen routing**

Replace `crates/duplff-gui/src/App.svelte`:
```svelte
<script lang="ts">
  import { currentScreen } from './lib/stores';
  import Setup from './screens/Setup.svelte';
  import Progress from './screens/Progress.svelte';
  import Results from './screens/Results.svelte';
  import Detail from './screens/Detail.svelte';
  import './app.css';
</script>

<main class="min-h-screen">
  {#if $currentScreen === 'setup'}
    <Setup />
  {:else if $currentScreen === 'progress'}
    <Progress />
  {:else if $currentScreen === 'results'}
    <Results />
  {:else if $currentScreen === 'detail'}
    <Detail />
  {/if}
</main>
```

**Step 8: Create placeholder screen files**

Create these four files under `crates/duplff-gui/src/screens/`:

`Setup.svelte`:
```svelte
<div class="p-8"><h1 class="text-2xl font-bold">Scan Setup</h1></div>
```

`Progress.svelte`:
```svelte
<div class="p-8"><h1 class="text-2xl font-bold">Scanning...</h1></div>
```

`Results.svelte`:
```svelte
<div class="p-8"><h1 class="text-2xl font-bold">Results</h1></div>
```

`Detail.svelte`:
```svelte
<div class="p-8"><h1 class="text-2xl font-bold">Group Detail</h1></div>
```

**Step 9: Verify it builds**

```bash
cd crates/duplff-gui && npm run tauri dev
```

Expected: Window shows "Scan Setup" with dark background.

**Step 10: Commit**

```bash
git add crates/duplff-gui/src/
git commit -m "feat: add frontend types, API, stores, and screen routing"
```

---

### Task 4: Setup Screen with FolderPicker

**Files:**
- Create: `crates/duplff-gui/src/components/FolderPicker.svelte`
- Modify: `crates/duplff-gui/src/screens/Setup.svelte`

**Step 1: Create FolderPicker component**

Create `crates/duplff-gui/src/components/FolderPicker.svelte`:
```svelte
<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';

  export let folders: string[] = [];
  export let label = 'Folders';

  async function addFolder() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: `Select ${label.toLowerCase()}`,
    });
    if (selected && !folders.includes(selected as string)) {
      folders = [...folders, selected as string];
    }
  }

  function removeFolder(index: number) {
    folders = folders.filter((_, i) => i !== index);
  }
</script>

<div class="space-y-2">
  <label class="text-sm font-medium text-gray-300">{label}</label>
  <div class="space-y-1">
    {#each folders as folder, i}
      <div class="flex items-center gap-2 bg-gray-800 rounded px-3 py-1.5 text-sm">
        <span class="truncate flex-1 font-mono">{folder}</span>
        <button
          class="text-gray-500 hover:text-delete shrink-0"
          on:click={() => removeFolder(i)}
        >×</button>
      </div>
    {/each}
  </div>
  <button
    class="w-full border-2 border-dashed border-gray-600 rounded-lg py-3 text-gray-400 hover:border-active hover:text-active transition-colors"
    on:click={addFolder}
  >
    + Add folder
  </button>
</div>
```

**Step 2: Implement Setup screen**

Replace `crates/duplff-gui/src/screens/Setup.svelte`:
```svelte
<script lang="ts">
  import { scanConfig, currentScreen } from '../lib/stores';
  import { startScan } from '../lib/api';
  import FolderPicker from '../components/FolderPicker.svelte';

  let roots: string[] = [];
  let priorityPaths: string[] = [];
  let extensions = '';
  let excludePatterns = '';
  let minSize = 1;
  let minSizeUnit: 'B' | 'KB' | 'MB' = 'KB';
  let maxSize = '';
  let followSymlinks = false;
  let paranoid = false;
  let noCache = false;

  function getMinSizeBytes(): number {
    const multipliers = { B: 1, KB: 1024, MB: 1024 * 1024 };
    return minSize * multipliers[minSizeUnit];
  }

  async function handleScan() {
    if (roots.length === 0) return;
    const config = {
      roots,
      extensions: extensions.trim()
        ? extensions.split(',').map(e => e.trim())
        : null,
      min_size: getMinSizeBytes(),
      max_size: maxSize ? parseInt(maxSize) * 1024 * 1024 : null,
      priority_paths: priorityPaths,
      follow_symlinks: followSymlinks,
      exclude_patterns: excludePatterns.trim()
        ? excludePatterns.split(',').map(p => p.trim())
        : [],
      no_cache: noCache,
      paranoid,
    };
    scanConfig.set(config);
    currentScreen.set('progress');
    await startScan(config);
  }
</script>

<div class="max-w-2xl mx-auto p-8">
  <h1 class="text-3xl font-bold mb-2">duplff</h1>
  <p class="text-gray-400 mb-8">Find and remove duplicate files</p>

  <div class="space-y-6">
    <FolderPicker bind:folders={roots} label="Scan directories" />

    <div>
      <label class="text-sm font-medium text-gray-300">Extension filter</label>
      <input
        type="text"
        bind:value={extensions}
        placeholder="e.g. py,rs,js (leave empty for all)"
        class="mt-1 w-full bg-gray-800 border border-gray-700 rounded px-3 py-2 text-sm focus:border-active focus:outline-none"
      />
    </div>

    <div>
      <label class="text-sm font-medium text-gray-300">Exclude patterns</label>
      <input
        type="text"
        bind:value={excludePatterns}
        placeholder="e.g. node_modules,.git,target"
        class="mt-1 w-full bg-gray-800 border border-gray-700 rounded px-3 py-2 text-sm focus:border-active focus:outline-none"
      />
    </div>

    <div class="grid grid-cols-2 gap-4">
      <div>
        <label class="text-sm font-medium text-gray-300">Min size</label>
        <div class="flex gap-2 mt-1">
          <input
            type="number"
            bind:value={minSize}
            min="0"
            class="flex-1 bg-gray-800 border border-gray-700 rounded px-3 py-2 text-sm focus:border-active focus:outline-none"
          />
          <select bind:value={minSizeUnit} class="bg-gray-800 border border-gray-700 rounded px-2 text-sm">
            <option value="B">B</option>
            <option value="KB">KB</option>
            <option value="MB">MB</option>
          </select>
        </div>
      </div>
      <div>
        <label class="text-sm font-medium text-gray-300">Max size (MB, optional)</label>
        <input
          type="number"
          bind:value={maxSize}
          placeholder="No limit"
          class="mt-1 w-full bg-gray-800 border border-gray-700 rounded px-3 py-2 text-sm focus:border-active focus:outline-none"
        />
      </div>
    </div>

    <FolderPicker bind:folders={priorityPaths} label="Priority directories (optional)" />

    <div class="space-y-2">
      <label class="flex items-center gap-2 text-sm">
        <input type="checkbox" bind:checked={followSymlinks} class="rounded" />
        <span>Follow symlinks</span>
      </label>
      <label class="flex items-center gap-2 text-sm">
        <input type="checkbox" bind:checked={paranoid} class="rounded" />
        <span>Paranoid mode (byte-by-byte verification)</span>
      </label>
      <label class="flex items-center gap-2 text-sm">
        <input type="checkbox" bind:checked={noCache} class="rounded" />
        <span>Disable hash cache</span>
      </label>
    </div>

    <button
      on:click={handleScan}
      disabled={roots.length === 0}
      class="w-full bg-active hover:bg-cyan-500 disabled:bg-gray-700 disabled:text-gray-500 text-white font-medium py-3 rounded-lg transition-colors"
    >
      Scan for duplicates
    </button>
  </div>
</div>
```

**Step 3: Verify the Setup screen**

```bash
cd crates/duplff-gui && npm run tauri dev
```

Expected: Dark-themed setup form with folder picker, inputs, checkboxes, and scan button.

**Step 4: Commit**

```bash
git add crates/duplff-gui/src/
git commit -m "feat: add Setup screen with folder picker and config form"
```

---

### Task 5: Progress Screen

**Files:**
- Modify: `crates/duplff-gui/src/screens/Progress.svelte`

**Step 1: Implement Progress screen**

Replace `crates/duplff-gui/src/screens/Progress.svelte`:
```svelte
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { currentScreen, report } from '../lib/stores';
  import { onScanProgress, onHashProgress, onScanComplete, onScanError } from '../lib/api';
  import type { UnlistenFn } from '@tauri-apps/api/event';

  let phase: 'scanning' | 'hashing' = 'scanning';
  let filesFound = 0;
  let hashDone = 0;
  let hashTotal = 0;
  let elapsed = 0;
  let error: string | null = null;
  let timer: ReturnType<typeof setInterval>;
  let unlisteners: UnlistenFn[] = [];

  onMount(async () => {
    const startTime = Date.now();
    timer = setInterval(() => {
      elapsed = Math.floor((Date.now() - startTime) / 1000);
    }, 1000);

    unlisteners.push(
      await onScanProgress((count) => { filesFound = count; }),
      await onHashProgress((progress) => {
        phase = 'hashing';
        hashDone = progress.done;
        hashTotal = progress.total;
      }),
      await onScanComplete((r) => {
        report.set(r);
        currentScreen.set('results');
      }),
      await onScanError((msg) => { error = msg; }),
    );
  });

  onDestroy(() => {
    clearInterval(timer);
    unlisteners.forEach(fn => fn());
  });

  function handleCancel() {
    currentScreen.set('setup');
  }

  function formatTime(secs: number): string {
    const m = Math.floor(secs / 60);
    const s = secs % 60;
    return m > 0 ? `${m}m ${s}s` : `${s}s`;
  }

  $: progressPct = hashTotal > 0 ? Math.round((hashDone / hashTotal) * 100) : 0;
</script>

<div class="max-w-lg mx-auto p-8 mt-20">
  <h2 class="text-2xl font-bold mb-6">
    {#if error}
      Scan failed
    {:else if phase === 'scanning'}
      Scanning files...
    {:else}
      Hashing files...
    {/if}
  </h2>

  {#if error}
    <div class="bg-red-900/30 border border-delete rounded-lg p-4 mb-6">
      <p class="text-delete">{error}</p>
    </div>
    <button on:click={handleCancel} class="w-full bg-gray-700 hover:bg-gray-600 text-white py-2 rounded-lg">
      Back to setup
    </button>
  {:else}
    <div class="w-full bg-gray-800 rounded-full h-3 mb-6 overflow-hidden">
      {#if phase === 'hashing' && hashTotal > 0}
        <div class="bg-active h-full rounded-full transition-all duration-300" style="width: {progressPct}%"></div>
      {:else}
        <div class="bg-active h-full rounded-full animate-pulse w-full opacity-30"></div>
      {/if}
    </div>

    <div class="space-y-2 text-sm text-gray-400 mb-8">
      <div class="flex justify-between">
        <span>Files found</span>
        <span class="text-gray-200 font-mono">{filesFound.toLocaleString()}</span>
      </div>
      {#if phase === 'hashing'}
        <div class="flex justify-between">
          <span>Files hashed</span>
          <span class="text-gray-200 font-mono">{hashDone.toLocaleString()} / {hashTotal.toLocaleString()}</span>
        </div>
        <div class="flex justify-between">
          <span>Progress</span>
          <span class="text-gray-200 font-mono">{progressPct}%</span>
        </div>
      {/if}
      <div class="flex justify-between">
        <span>Elapsed</span>
        <span class="text-gray-200 font-mono">{formatTime(elapsed)}</span>
      </div>
    </div>

    <button on:click={handleCancel} class="w-full bg-gray-700 hover:bg-gray-600 text-white py-2 rounded-lg">
      Cancel
    </button>
  {/if}
</div>
```

**Step 2: Verify**

```bash
cd crates/duplff-gui && npm run tauri dev
```

Expected: After clicking "Scan" from Setup, Progress screen shows with progress bar and stats.

**Step 3: Commit**

```bash
git add crates/duplff-gui/src/screens/Progress.svelte
git commit -m "feat: add Progress screen with live event updates"
```

---

### Task 6: Results Screen with GroupTable

**Files:**
- Create: `crates/duplff-gui/src/components/GroupTable.svelte`
- Create: `crates/duplff-gui/src/components/ConfirmDialog.svelte`
- Modify: `crates/duplff-gui/src/screens/Results.svelte`

**Step 1: Create GroupTable component**

Create `crates/duplff-gui/src/components/GroupTable.svelte`:
```svelte
<script lang="ts">
  import type { DuplicateGroup, SortMode } from '../lib/types';
  import { formatBytes, truncatePath } from '../lib/format';

  export let groups: DuplicateGroup[];
  export let sortMode: SortMode;
  export let filterText: string;
  export let onSelectGroup: (index: number) => void;

  $: filteredGroups = groups
    .map((g, i) => ({ group: g, index: i }))
    .filter(({ group }) => {
      if (!filterText) return true;
      const lower = filterText.toLowerCase();
      return group.keep.entry.path.toLowerCase().includes(lower) ||
        group.duplicates.some(d => d.entry.path.toLowerCase().includes(lower));
    });

  $: sortedGroups = [...filteredGroups].sort((a, b) => {
    switch (sortMode) {
      case 'wasted':
        return (b.group.size * b.group.duplicates.length) - (a.group.size * a.group.duplicates.length);
      case 'size':
        return b.group.size - a.group.size;
      case 'files':
        return (b.group.duplicates.length + 1) - (a.group.duplicates.length + 1);
      case 'path':
        return a.group.keep.entry.path.localeCompare(b.group.keep.entry.path);
      default:
        return 0;
    }
  });
</script>

<div class="overflow-auto">
  <table class="w-full text-sm">
    <thead class="text-gray-400 text-left border-b border-gray-700">
      <tr>
        <th class="py-2 px-3 w-12">#</th>
        <th class="py-2 px-3">Files</th>
        <th class="py-2 px-3">Size</th>
        <th class="py-2 px-3">Wasted</th>
        <th class="py-2 px-3">Sample Path</th>
      </tr>
    </thead>
    <tbody>
      {#each sortedGroups as { group, index }, i}
        <tr
          class="border-b border-gray-800 hover:bg-gray-800/50 cursor-pointer transition-colors"
          on:click={() => onSelectGroup(index)}
        >
          <td class="py-2 px-3 text-gray-500">{i + 1}</td>
          <td class="py-2 px-3 font-mono">{group.duplicates.length + 1}</td>
          <td class="py-2 px-3 font-mono">{formatBytes(group.size)}</td>
          <td class="py-2 px-3 font-mono text-delete">{formatBytes(group.size * group.duplicates.length)}</td>
          <td class="py-2 px-3 font-mono truncate max-w-xs" title={group.keep.entry.path}>
            {truncatePath(group.keep.entry.path)}
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>
```

**Step 2: Create ConfirmDialog component**

Create `crates/duplff-gui/src/components/ConfirmDialog.svelte`:
```svelte
<script lang="ts">
  export let title: string;
  export let message: string;
  export let onConfirm: () => void;
  export let onCancel: () => void;
</script>

<div class="fixed inset-0 bg-black/60 flex items-center justify-center z-50">
  <div class="bg-gray-800 border border-gray-700 rounded-lg p-6 max-w-md w-full mx-4 shadow-xl">
    <h3 class="text-lg font-bold mb-2">{title}</h3>
    <p class="text-gray-400 text-sm mb-6">{message}</p>
    <div class="flex gap-3 justify-end">
      <button on:click={onCancel} class="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded text-sm">
        Cancel
      </button>
      <button on:click={onConfirm} class="px-4 py-2 bg-delete hover:bg-red-600 rounded text-sm">
        Trash
      </button>
    </div>
  </div>
</div>
```

**Step 3: Implement Results screen**

Replace `crates/duplff-gui/src/screens/Results.svelte`:
```svelte
<script lang="ts">
  import { report, currentScreen, selectedGroup, markedFiles, filterText, sortMode } from '../lib/stores';
  import { trashFiles, exportJson, exportCsv, undoLast, getResults } from '../lib/api';
  import { formatBytes } from '../lib/format';
  import GroupTable from '../components/GroupTable.svelte';
  import ConfirmDialog from '../components/ConfirmDialog.svelte';
  import type { SortMode } from '../lib/types';

  let showConfirm = false;
  let statusMessage: string | null = null;

  $: groups = $report?.groups ?? [];
  $: totalDuplicates = $report?.total_duplicates ?? 0;
  $: totalWasted = $report?.total_wasted_bytes ?? 0;

  function selectGroup(index: number) {
    selectedGroup.set(index);
    currentScreen.set('detail');
  }

  function autoSelectAll() {
    const paths = new Set<string>();
    for (const group of groups) {
      for (const dup of group.duplicates) {
        paths.add(dup.entry.path);
      }
    }
    markedFiles.set(paths);
  }

  function clearSelection() {
    markedFiles.set(new Set());
  }

  async function handleTrash() {
    const paths = Array.from($markedFiles);
    if (paths.length === 0) return;
    showConfirm = false;
    try {
      const result = await trashFiles(paths);
      statusMessage = `Trashed ${result.count} files (${formatBytes(result.bytes_reclaimed)})`;
      markedFiles.set(new Set());
      const updated = await getResults();
      if (updated) report.set(updated);
    } catch (e) {
      statusMessage = `Error: ${e}`;
    }
  }

  async function handleUndo() {
    try {
      const result = await undoLast();
      statusMessage = `Restored ${result.restored} files`;
    } catch (e) {
      statusMessage = `Undo failed: ${e}`;
    }
  }

  async function handleExportJson() {
    const data = await exportJson();
    downloadFile('duplff-report.json', data, 'application/json');
  }

  async function handleExportCsv() {
    const data = await exportCsv();
    downloadFile('duplff-report.csv', data, 'text/csv');
  }

  function downloadFile(name: string, content: string, type: string) {
    const blob = new Blob([content], { type });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = name;
    a.click();
    URL.revokeObjectURL(url);
  }

  function handleSort(mode: SortMode) {
    sortMode.set(mode);
  }

  function newScan() {
    report.set(null);
    markedFiles.set(new Set());
    currentScreen.set('setup');
  }
</script>

<div class="flex flex-col h-screen">
  <div class="bg-gray-800 border-b border-gray-700 px-6 py-3 flex items-center justify-between">
    <div class="flex items-center gap-6 text-sm">
      <span>{groups.length} groups</span>
      <span>{totalDuplicates} duplicates</span>
      <span class="text-delete">{formatBytes(totalWasted)} wasted</span>
    </div>
    <button on:click={newScan} class="text-sm text-gray-400 hover:text-white px-3 py-1">
      New Scan
    </button>
  </div>

  <div class="px-6 py-3 flex items-center gap-3 border-b border-gray-800">
    <input
      type="text"
      placeholder="Filter by path..."
      bind:value={$filterText}
      class="bg-gray-800 border border-gray-700 rounded px-3 py-1.5 text-sm flex-1 focus:border-active focus:outline-none"
    />
    <select
      value={$sortMode}
      on:change={(e) => handleSort(e.currentTarget.value)}
      class="bg-gray-800 border border-gray-700 rounded px-3 py-1.5 text-sm"
    >
      <option value="wasted">Sort: Wasted</option>
      <option value="size">Sort: Size</option>
      <option value="files">Sort: Files</option>
      <option value="path">Sort: Path</option>
    </select>
  </div>

  <div class="flex-1 overflow-auto">
    <GroupTable {groups} sortMode={$sortMode} filterText={$filterText} onSelectGroup={selectGroup} />
  </div>

  <div class="bg-gray-800 border-t border-gray-700 px-6 py-3 flex items-center justify-between">
    <div class="flex gap-2">
      <button on:click={autoSelectAll} class="text-sm bg-gray-700 hover:bg-gray-600 px-3 py-1.5 rounded">
        Select All Duplicates
      </button>
      <button on:click={clearSelection} class="text-sm bg-gray-700 hover:bg-gray-600 px-3 py-1.5 rounded">
        Clear Selection
      </button>
      {#if $markedFiles.size > 0}
        <button on:click={() => showConfirm = true} class="text-sm bg-delete hover:bg-red-600 px-3 py-1.5 rounded">
          Trash {$markedFiles.size} files
        </button>
      {/if}
    </div>
    <div class="flex gap-2">
      <button on:click={handleUndo} class="text-sm bg-gray-700 hover:bg-gray-600 px-3 py-1.5 rounded">Undo</button>
      <button on:click={handleExportJson} class="text-sm bg-gray-700 hover:bg-gray-600 px-3 py-1.5 rounded">JSON</button>
      <button on:click={handleExportCsv} class="text-sm bg-gray-700 hover:bg-gray-600 px-3 py-1.5 rounded">CSV</button>
    </div>
  </div>

  {#if statusMessage}
    <div class="bg-gray-800 border-t border-gray-700 px-6 py-2 text-sm text-gray-400">
      {statusMessage}
    </div>
  {/if}
</div>

{#if showConfirm}
  <ConfirmDialog
    title="Trash files?"
    message="Move {$markedFiles.size} files to the OS trash? This can be undone."
    onConfirm={handleTrash}
    onCancel={() => showConfirm = false}
  />
{/if}
```

**Step 4: Verify**

```bash
cd crates/duplff-gui && npm run tauri dev
```

**Step 5: Commit**

```bash
git add crates/duplff-gui/src/
git commit -m "feat: add Results screen with GroupTable, sort, filter, and actions"
```

---

### Task 7: Detail Screen with FileList

**Files:**
- Create: `crates/duplff-gui/src/components/FileList.svelte`
- Modify: `crates/duplff-gui/src/screens/Detail.svelte`

**Step 1: Create FileList component**

Create `crates/duplff-gui/src/components/FileList.svelte`:
```svelte
<script lang="ts">
  import type { RankedFile } from '../lib/types';
  import { formatBytes, reasonLabel } from '../lib/format';
  import { openInFileManager } from '../lib/api';

  export let keep: RankedFile;
  export let duplicates: RankedFile[];
  export let markedPaths: Set<string>;
  export let onToggle: (path: string) => void;
</script>

<div class="space-y-1">
  <div class="flex items-center gap-3 bg-gray-800 rounded-lg px-4 py-3">
    <span class="text-keep text-lg w-5 text-center">✓</span>
    <div class="flex-1 min-w-0">
      <p class="font-mono text-sm truncate" title={keep.entry.path}>{keep.entry.path}</p>
      <p class="text-xs text-gray-500">{formatBytes(keep.entry.size)}</p>
    </div>
    <span class="text-xs bg-keep/20 text-keep px-2 py-0.5 rounded shrink-0">
      Kept: {reasonLabel(keep.reason)}
    </span>
    <button
      on:click={() => openInFileManager(keep.entry.path)}
      class="text-xs text-gray-500 hover:text-active shrink-0"
    >Open</button>
  </div>

  {#each duplicates as dup}
    <div class="flex items-center gap-3 bg-gray-800/50 rounded-lg px-4 py-3">
      <input
        type="checkbox"
        checked={markedPaths.has(dup.entry.path)}
        on:change={() => onToggle(dup.entry.path)}
        class="w-5 h-5 rounded"
      />
      <div class="flex-1 min-w-0">
        <p class="font-mono text-sm truncate" title={dup.entry.path}>{dup.entry.path}</p>
        <p class="text-xs text-gray-500">{formatBytes(dup.entry.size)}</p>
      </div>
      <span class="text-xs bg-delete/20 text-delete px-2 py-0.5 rounded shrink-0">Duplicate</span>
      <button
        on:click={() => openInFileManager(dup.entry.path)}
        class="text-xs text-gray-500 hover:text-active shrink-0"
      >Open</button>
    </div>
  {/each}
</div>
```

**Step 2: Implement Detail screen**

Replace `crates/duplff-gui/src/screens/Detail.svelte`:
```svelte
<script lang="ts">
  import { report, currentScreen, selectedGroup, markedFiles } from '../lib/stores';
  import { trashFiles, undoLast, getResults } from '../lib/api';
  import { formatBytes } from '../lib/format';
  import FileList from '../components/FileList.svelte';
  import ConfirmDialog from '../components/ConfirmDialog.svelte';

  let showConfirm = false;
  let statusMessage: string | null = null;

  $: group = $report?.groups[$selectedGroup] ?? null;
  $: groupMarked = group
    ? new Set([...$markedFiles].filter(p => group!.duplicates.some(d => d.entry.path === p)))
    : new Set<string>();

  function toggleFile(path: string) {
    markedFiles.update(set => {
      const next = new Set(set);
      if (next.has(path)) next.delete(path);
      else next.add(path);
      return next;
    });
  }

  function selectAllDuplicates() {
    if (!group) return;
    markedFiles.update(set => {
      const next = new Set(set);
      for (const dup of group!.duplicates) next.add(dup.entry.path);
      return next;
    });
  }

  function deselectAll() {
    if (!group) return;
    markedFiles.update(set => {
      const next = new Set(set);
      for (const dup of group!.duplicates) next.delete(dup.entry.path);
      return next;
    });
  }

  async function handleTrash() {
    showConfirm = false;
    const paths = Array.from(groupMarked);
    try {
      const result = await trashFiles(paths);
      statusMessage = `Trashed ${result.count} files (${formatBytes(result.bytes_reclaimed)})`;
      markedFiles.update(set => {
        const next = new Set(set);
        for (const p of paths) next.delete(p);
        return next;
      });
      const updated = await getResults();
      if (updated) report.set(updated);
    } catch (e) {
      statusMessage = `Error: ${e}`;
    }
  }

  async function handleUndo() {
    try {
      const result = await undoLast();
      statusMessage = `Restored ${result.restored} files`;
    } catch (e) {
      statusMessage = `Undo failed: ${e}`;
    }
  }
</script>

<div class="flex flex-col h-screen">
  <div class="bg-gray-800 border-b border-gray-700 px-6 py-3">
    <div class="flex items-center gap-4">
      <button on:click={() => currentScreen.set('results')} class="text-gray-400 hover:text-white text-sm">
        ← Back
      </button>
      {#if group}
        <div class="flex items-center gap-4 text-sm text-gray-400">
          <span>Size: <span class="text-gray-200 font-mono">{formatBytes(group.size)}</span></span>
          <span>Files: <span class="text-gray-200 font-mono">{group.duplicates.length + 1}</span></span>
          <span class="text-delete">Wasted: <span class="font-mono">{formatBytes(group.size * group.duplicates.length)}</span></span>
        </div>
      {/if}
    </div>
  </div>

  <div class="flex-1 overflow-auto p-6">
    {#if group}
      <FileList keep={group.keep} duplicates={group.duplicates} markedPaths={groupMarked} onToggle={toggleFile} />
    {:else}
      <p class="text-gray-500">Group not found.</p>
    {/if}
  </div>

  <div class="bg-gray-800 border-t border-gray-700 px-6 py-3 flex items-center justify-between">
    <div class="flex gap-2">
      <button on:click={selectAllDuplicates} class="text-sm bg-gray-700 hover:bg-gray-600 px-3 py-1.5 rounded">Select All</button>
      <button on:click={deselectAll} class="text-sm bg-gray-700 hover:bg-gray-600 px-3 py-1.5 rounded">Deselect All</button>
    </div>
    <div class="flex gap-2">
      <button on:click={handleUndo} class="text-sm bg-gray-700 hover:bg-gray-600 px-3 py-1.5 rounded">Undo</button>
      {#if groupMarked.size > 0}
        <button on:click={() => showConfirm = true} class="text-sm bg-delete hover:bg-red-600 px-3 py-1.5 rounded">
          Trash {groupMarked.size} files
        </button>
      {/if}
    </div>
  </div>

  {#if statusMessage}
    <div class="bg-gray-800 border-t border-gray-700 px-6 py-2 text-sm text-gray-400">{statusMessage}</div>
  {/if}
</div>

{#if showConfirm}
  <ConfirmDialog
    title="Trash selected files?"
    message="Move {groupMarked.size} files to the OS trash? This can be undone."
    onConfirm={handleTrash}
    onCancel={() => showConfirm = false}
  />
{/if}
```

**Step 3: Verify**

```bash
cd crates/duplff-gui && npm run tauri dev
```

Expected: Click a group in Results → Detail shows files with checkboxes, keep badge, and action buttons.

**Step 4: Commit**

```bash
git add crates/duplff-gui/src/
git commit -m "feat: add Detail screen with FileList and trash actions"
```

---

### Task 8: Integration Testing and Polish

**Files:**
- Various files in `crates/duplff-gui/`

**Step 1: Test full workflow end-to-end**

```bash
cd crates/duplff-gui && npm run tauri dev
```

Test the complete flow:
1. Add folders on Setup screen
2. Click Scan → see Progress with live updates
3. Results show with correct group count and wasted bytes
4. Click a group → Detail shows files correctly
5. Mark duplicates → Trash → Confirm → Files removed
6. Undo → Files restored
7. Export JSON → Downloads file
8. Export CSV → Downloads file
9. Filter on Results works
10. Sort on Results works
11. New Scan from Results works

**Step 2: Fix any issues found during testing**

Address compilation errors, missing imports, visual bugs, or broken interactions.

**Step 3: Run clippy on the Rust backend**

```bash
export PATH="$HOME/.cargo/bin:$PATH"
cd crates/duplff-gui/src-tauri && cargo clippy -- -D warnings
```

**Step 4: Build release binary**

```bash
cd crates/duplff-gui && npm run tauri build
```

Verify the binary is produced and the app launches from the built binary.

**Step 5: Commit any fixes**

```bash
git add crates/duplff-gui/
git commit -m "fix: integration testing fixes and polish"
```

---

## Summary

| Task | Description | Key Files |
|------|-------------|-----------|
| 1 | Scaffold Tauri + Svelte + Tailwind | `crates/duplff-gui/`, workspace `Cargo.toml` |
| 2 | Tauri backend: state, progress, all commands | `state.rs`, `commands.rs`, `main.rs` |
| 3 | Frontend types, API, stores, routing | `types.ts`, `api.ts`, `stores.ts`, `App.svelte` |
| 4 | Setup screen + FolderPicker | `Setup.svelte`, `FolderPicker.svelte` |
| 5 | Progress screen | `Progress.svelte` |
| 6 | Results screen + GroupTable | `Results.svelte`, `GroupTable.svelte`, `ConfirmDialog.svelte` |
| 7 | Detail screen + FileList | `Detail.svelte`, `FileList.svelte` |
| 8 | Integration testing + polish | Various |
