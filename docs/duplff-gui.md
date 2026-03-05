# duplff-gui

Desktop application for duplff built with Tauri v2 and SvelteKit.

## Screens

### Setup

Configure a scan:

- **Scan directories** — Add one or more folders to scan.
- **Min / Max size** — Filter by file size.
- **Extensions** — Comma-separated list (e.g. `py, rs, js`). Leave empty to scan all.
- **Advanced** — Toggle to reveal exclude patterns, priority directories, paranoid mode, cache control, and symlink following.

### Progress

Shows real-time scan progress:

- File discovery count during the scanning phase.
- Hash progress bar during the hashing phase.
- Elapsed time.

### Results

Table of duplicate groups:

- Sortable by wasted space, file size, file count, or path.
- Filter by path substring.
- **Select All** marks every duplicate across all groups.
- **Trash** moves selected files to the OS trash.
- **Undo** restores the last trashed batch.
- **Export** as JSON or CSV.

### Detail

View all files in a single duplicate group:

- The kept file is marked with a green border.
- Duplicates have checkboxes for selection.
- Hover to reveal the open-in-file-manager button.
- Trash and undo operate on the current group's selection.

## Backend Commands

The Tauri backend exposes these commands:

| Command | Description |
|---------|-------------|
| `start_scan` | Begin scanning with a `ScanConfig` |
| `get_results` | Fetch the current `DuplicateReport` |
| `trash_files` | Move selected files to OS trash |
| `undo_last` | Restore the last trashed batch |
| `dry_run` | Preview what would be deleted |
| `list_action_logs` | List all past trash operations |
| `undo_log` | Undo a specific past operation |
| `export_json` | Export report as JSON |
| `export_csv` | Export report as CSV |
| `open_in_file_manager` | Open a file's parent directory |

## Building

Requires system dependencies on Linux:

```
sudo apt-get install libwebkit2gtk-4.1-dev build-essential libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

Then:

```
cd crates/duplff-gui
npm ci
npx tauri build
```

Output artifacts are in `target/release/bundle/` (.deb and .AppImage).
