# GUI Polish, Feature Integration & CI/CD Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Integrate remaining duplff-core features into the GUI, redesign the UI to be clean and native-feeling, and add a GitHub Actions CI/CD pipeline with automated releases.

**Architecture:** Three independent workstreams: (1) backend commands + frontend wiring for missing features, (2) full UI redesign of all Svelte screens with refined Tailwind styling, (3) GitHub Actions workflow for test + build + release.

**Tech Stack:** Rust (Tauri v2), Svelte 5, Tailwind CSS v4, GitHub Actions, `tauri-action`

---

## Objective 1: Feature Integration

### Task 1: Add Missing Backend Commands

**Files:**
- Modify: `crates/duplff-gui/src-tauri/src/commands.rs`
- Modify: `crates/duplff-gui/src-tauri/src/lib.rs`

**Step 1: Add dry_run command**

Add to `commands.rs`:
```rust
#[derive(Serialize)]
pub struct DryRunPlan {
    pub files_to_delete: Vec<String>,
    pub bytes_to_reclaim: u64,
    pub group_count: usize,
}

#[tauri::command]
pub fn dry_run(state: State<'_, AppState>) -> Result<DryRunPlan, String> {
    let report = state.report.lock().unwrap();
    let report = report.as_ref().ok_or("no results")?;
    let plan = duplff_core::actions::dry_run(&report.groups);
    Ok(DryRunPlan {
        files_to_delete: plan.files_to_delete.iter().map(|p| p.display().to_string()).collect(),
        bytes_to_reclaim: plan.bytes_to_reclaim,
        group_count: report.groups.len(),
    })
}
```

**Step 2: Add action log commands**

```rust
#[derive(Serialize)]
pub struct ActionLogSummary {
    pub timestamp: String,
    pub file_count: usize,
    pub bytes_reclaimed: u64,
}

#[tauri::command]
pub fn list_action_logs() -> Result<Vec<ActionLogSummary>, String> {
    let paths = duplff_core::log_store::list_logs().map_err(|e| e.to_string())?;
    let mut summaries = Vec::new();
    for path in paths {
        if let Ok(log) = duplff_core::log_store::load_log(&path) {
            summaries.push(ActionLogSummary {
                timestamp: log.timestamp.clone(),
                file_count: log.actions.len(),
                bytes_reclaimed: log.bytes_reclaimed,
            });
        }
    }
    Ok(summaries)
}

#[tauri::command]
pub fn undo_log(timestamp: String) -> Result<UndoResult, String> {
    let paths = duplff_core::log_store::list_logs().map_err(|e| e.to_string())?;
    for path in paths {
        if let Ok(log) = duplff_core::log_store::load_log(&path) {
            if log.timestamp == timestamp {
                let result = duplff_core::actions::undo(&log).map_err(|e| e.to_string())?;
                return Ok(UndoResult {
                    restored: result.restored.len(),
                    not_found: result.not_found.len(),
                });
            }
        }
    }
    Err("log not found".into())
}
```

**Step 3: Add CSV export with keep_reason field (match CLI format)**

Update the existing `export_csv` to include `keep_reason`:
```rust
#[tauri::command]
pub fn export_csv(state: State<'_, AppState>) -> Result<String, String> {
    let report = state.report.lock().unwrap();
    let report = report.as_ref().ok_or("no results to export")?;
    let mut wtr = csv::Writer::from_writer(vec![]);
    wtr.write_record(["Group", "Hash", "Path", "Size", "Status", "Reason"])
        .map_err(|e| e.to_string())?;
    for (i, group) in report.groups.iter().enumerate() {
        let hash = hex::encode(group.hash);
        wtr.write_record([
            &(i + 1).to_string(), &hash,
            &group.keep.entry.path.display().to_string(),
            &group.size.to_string(), "keep",
            &format!("{}", group.keep.reason),
        ]).map_err(|e| e.to_string())?;
        for dup in &group.duplicates {
            wtr.write_record([
                &(i + 1).to_string(), &hash,
                &dup.entry.path.display().to_string(),
                &group.size.to_string(), "duplicate",
                &format!("{}", dup.reason),
            ]).map_err(|e| e.to_string())?;
        }
    }
    String::from_utf8(wtr.into_inner().map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}
```

**Step 4: Register new commands in lib.rs**

Add `commands::dry_run`, `commands::list_action_logs`, `commands::undo_log` to the `generate_handler!` macro.

**Step 5: Verify compilation**

```bash
export PATH="$HOME/.cargo/bin:$PATH" && cargo clippy -p duplff-gui -- -D warnings
```

**Step 6: Commit**

```bash
git add crates/duplff-gui/src-tauri/
git commit -m "feat: add dry_run, action log, and improved CSV commands"
```

---

### Task 2: Wire New Commands to Frontend

**Files:**
- Modify: `crates/duplff-gui/src/lib/types.ts`
- Modify: `crates/duplff-gui/src/lib/api.ts`

**Step 1: Add types**

Add to `types.ts`:
```typescript
export interface DryRunPlan {
  files_to_delete: string[];
  bytes_to_reclaim: number;
  group_count: number;
}

export interface ActionLogSummary {
  timestamp: string;
  file_count: number;
  bytes_reclaimed: number;
}
```

**Step 2: Add API functions**

Add to `api.ts`:
```typescript
export async function dryRun(): Promise<DryRunPlan> {
  return invoke('dry_run');
}

export async function listActionLogs(): Promise<ActionLogSummary[]> {
  return invoke('list_action_logs');
}

export async function undoLog(timestamp: string): Promise<UndoResult> {
  return invoke('undo_log', { timestamp });
}
```

**Step 3: Verify build**

```bash
cd crates/duplff-gui && npm run build
```

**Step 4: Commit**

```bash
git add crates/duplff-gui/src/lib/
git commit -m "feat: add dry_run and action log API types"
```

---

## Objective 2: UI Redesign

### Task 3: Redesign Setup Screen

The Setup screen should feel native and minimal. Remove unnecessary labels, use placeholder text, group related controls visually.

**Files:**
- Modify: `crates/duplff-gui/src/lib/screens/Setup.svelte`
- Modify: `crates/duplff-gui/src/lib/components/FolderPicker.svelte`
- Modify: `crates/duplff-gui/src/app.css`

**Step 1: Redesign Setup.svelte**

Replace the entire file. Key design principles:
- Clean card layout with subtle borders
- Collapsible "Advanced" section for less-used options (paranoid, no-cache, symlinks)
- FolderPicker with visual drop-zone feel
- Compact form with smart defaults
- Scan button is the clear primary CTA
- Remove redundant label text — use placeholders and icons where possible
- Add `total_files_scanned` and `total_bytes_scanned` from previous scans if available

The subagent implementing this should:
- Read the current Setup.svelte
- Redesign it with a clean, native dark theme
- Use Tailwind utility classes for all styling
- Keep the existing data flow (stores, startScan)
- Add an "Advanced" toggle that shows/hides paranoid, no-cache, follow-symlinks, exclude patterns
- Make the FolderPicker more visually prominent with a larger drop-zone area

**Step 2: Redesign FolderPicker.svelte**

- Larger add button with folder icon (use Unicode or SVG)
- Selected folders shown as compact chips with hover-to-remove
- More visual feedback on hover

**Step 3: Update app.css theme**

Add refined base styles:
```css
* {
  box-sizing: border-box;
}

::-webkit-scrollbar {
  width: 6px;
}
::-webkit-scrollbar-track {
  background: transparent;
}
::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 3px;
}
::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.2);
}
```

**Step 4: Verify**

```bash
cd crates/duplff-gui && npm run build
```

**Step 5: Commit**

```bash
git add crates/duplff-gui/src/
git commit -m "feat: redesign Setup screen with clean native UI"
```

---

### Task 4: Redesign Progress Screen

**Files:**
- Modify: `crates/duplff-gui/src/lib/screens/Progress.svelte`

**Step 1: Redesign Progress.svelte**

Key design principles:
- Centered layout with a single clean card
- Subtle animated progress bar (not pulsing — use smooth CSS transitions)
- Minimal stats: files found, progress %, elapsed
- Phase indicator as small text above the progress bar
- Remove visual clutter — no boxes around error messages, just red text
- Cancel button is understated (ghost button)

**Step 2: Verify and commit**

```bash
cd crates/duplff-gui && npm run build
git add crates/duplff-gui/src/lib/screens/Progress.svelte
git commit -m "feat: redesign Progress screen with clean animations"
```

---

### Task 5: Redesign Results Screen

**Files:**
- Modify: `crates/duplff-gui/src/lib/screens/Results.svelte`
- Modify: `crates/duplff-gui/src/lib/components/GroupTable.svelte`
- Modify: `crates/duplff-gui/src/lib/components/ConfirmDialog.svelte`

**Step 1: Redesign Results.svelte**

Key design principles:
- Summary bar: compact with 3 key metrics (groups, duplicates, wasted space)
- Table: clean with minimal borders, zebra striping with very subtle alternating rows
- Sort controls: clickable column headers with sort direction indicators (▲▼)
- Filter: integrated into the header as a search icon that expands
- Action bar: sticky bottom with clear visual hierarchy
- Status messages: toast-style notifications that auto-dismiss after 3 seconds
- "Trash" button changes to show the deletion preview count + bytes
- Remove button labels where icons suffice

**Step 2: Redesign GroupTable.svelte**

- Hover effect with subtle background change
- Active/selected row highlight
- Wasted column uses proportional bar visualization (thin colored bar inside the cell)
- Clean typography with monospace only for numbers

**Step 3: Redesign ConfirmDialog.svelte**

- Frosted glass backdrop
- Clean card with clear action hierarchy
- Destructive action button is clearly marked
- Show file count and bytes in the dialog body

**Step 4: Verify and commit**

```bash
cd crates/duplff-gui && npm run build
git add crates/duplff-gui/src/
git commit -m "feat: redesign Results screen with clean table and toast notifications"
```

---

### Task 6: Redesign Detail Screen

**Files:**
- Modify: `crates/duplff-gui/src/lib/screens/Detail.svelte`
- Modify: `crates/duplff-gui/src/lib/components/FileList.svelte`

**Step 1: Redesign Detail.svelte**

Key design principles:
- Clean header with breadcrumb-style navigation (Results > Group N)
- Group stats displayed as compact badges in header
- File hash displayed as a subtle monospace string (truncated, copyable on click)
- Action bar: same style as Results for consistency

**Step 2: Redesign FileList.svelte**

- Keep file: subtle green left border accent (not a full green background)
- Duplicate files: neutral with checkbox
- Reason badge: small pill, not prominent
- "Open" button: folder icon, appears on hover only
- File paths: truncated from the left with full path in tooltip
- Compact vertical spacing — more files visible at once

**Step 3: Verify and commit**

```bash
cd crates/duplff-gui && npm run build
git add crates/duplff-gui/src/
git commit -m "feat: redesign Detail screen with clean file list"
```

---

## Objective 3: GitHub Actions CI/CD

### Task 7: Create CI Pipeline (Test + Build)

**Files:**
- Create: `.github/workflows/ci.yml`

**Step 1: Write the CI workflow**

Create `.github/workflows/ci.yml`:
```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy

      - name: Cache cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Run clippy
        run: cargo clippy --workspace -- -D warnings

      - name: Run tests
        run: cargo test --workspace

  build-gui:
    runs-on: ubuntu-latest
    needs: test
    steps:
      - uses: actions/checkout@v4

      - name: Install system dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install Node.js
        uses: actions/setup-node@v4
        with:
          node-version: 20

      - name: Cache cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Install frontend dependencies
        working-directory: crates/duplff-gui
        run: npm ci

      - name: Build GUI
        working-directory: crates/duplff-gui
        run: npm run tauri build

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: duplff-linux
          path: |
            crates/duplff-gui/src-tauri/target/release/bundle/deb/*.deb
            crates/duplff-gui/src-tauri/target/release/bundle/appimage/*.AppImage
```

**Step 2: Commit**

```bash
git add .github/
git commit -m "ci: add GitHub Actions workflow for test and build"
```

---

### Task 8: Add Release Workflow

**Files:**
- Create: `.github/workflows/release.yml`

**Step 1: Write the release workflow**

This workflow triggers on version tags (e.g., `v0.1.0`) and creates a GitHub Release with the built artifacts.

Create `.github/workflows/release.yml`:
```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

permissions:
  contents: write

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install system dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install Node.js
        uses: actions/setup-node@v4
        with:
          node-version: 20

      - name: Cache cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-release-${{ hashFiles('**/Cargo.lock') }}

      - name: Run tests
        run: cargo test --workspace

      - name: Install frontend dependencies
        working-directory: crates/duplff-gui
        run: npm ci

      - name: Build GUI (release)
        working-directory: crates/duplff-gui
        run: npm run tauri build

      - name: Build CLI (release)
        run: cargo build --release -p duplff-cli

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          generate_release_notes: true
          files: |
            crates/duplff-gui/src-tauri/target/release/bundle/deb/*.deb
            crates/duplff-gui/src-tauri/target/release/bundle/appimage/*.AppImage
            target/release/duplff
```

**Step 2: Commit**

```bash
git add .github/
git commit -m "ci: add release workflow with GitHub Releases"
```

---

### Task 9: Verify CI Pipeline

**Step 1: Push and check CI runs**

```bash
git push origin main
```

Check: `gh run list --limit 3`

**Step 2: Fix any CI failures**

If the CI workflow fails, debug and fix. Common issues:
- Missing `npm ci` needs `package-lock.json` committed
- WebKitGTK version mismatch on Ubuntu
- Tauri build requires specific Ubuntu version (22.04+)

**Step 3: Commit any fixes**

```bash
git add .
git commit -m "fix: CI pipeline adjustments"
```

---

## Summary

| Task | Objective | Description |
|------|-----------|-------------|
| 1 | Feature Integration | Add dry_run, action log, improved CSV backend commands |
| 2 | Feature Integration | Wire new commands to frontend types + API |
| 3 | UI Redesign | Redesign Setup screen — clean, native, collapsible advanced |
| 4 | UI Redesign | Redesign Progress screen — smooth animations, minimal |
| 5 | UI Redesign | Redesign Results screen — clean table, toasts, sort headers |
| 6 | UI Redesign | Redesign Detail screen — clean file list, hover actions |
| 7 | CI/CD | GitHub Actions CI pipeline (test + build) |
| 8 | CI/CD | Release workflow with GitHub Releases |
| 9 | CI/CD | Verify and fix CI pipeline |
