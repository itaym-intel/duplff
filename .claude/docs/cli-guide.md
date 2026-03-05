# duplff-cli Internals

## Clap Arguments (`cli.rs`)

```
duplff [OPTIONS] <PATHS>...

Positional:  PATHS...              Directories to scan
Options:
  -e, --ext EXTENSIONS             File extensions (repeatable)
  -m, --min-size MIN_SIZE          Min file size bytes (default: 1)
  -M, --max-size MAX_SIZE          Max file size bytes
  -p, --priority PRIORITY          Priority directories (repeatable)
  -x, --exclude EXCLUDE            Exclude patterns (repeatable)
  -L, --follow-symlinks            Follow symlinks
      --json                       JSON output (non-interactive)
      --dry-run                    Show plan (non-interactive)
      --csv                        CSV output (non-interactive)
      --no-cache                   Disable hash cache
      --paranoid                   Byte-by-byte verify
```

## Dispatch Logic (`main.rs`)

```
parse args → build ScanConfig
  → if --json  → non_interactive::run_json()
  → if --csv   → non_interactive::run_csv()
  → if --dry-run → non_interactive::run_dry_run()
  → else       → tui::run() (interactive TUI)
```

## TUI Architecture (`tui/`)

### State Machine (`state.rs`)

```
AppState enum:
  Scanning { files_found, files_hashed, total_to_hash, phase }
  Results  { report, group_cursor, detail_cursor, focus, marked, filter, sort }
  Confirm  { ...Results fields + message }
  Help     { ...Results fields underneath }
  Error    { message }
```

Focus panes: `Groups` | `Detail`
Sort modes: `WastedDesc` | `SizeDesc` | `FileCountDesc` | `PathAsc`

### Scan Thread (`scan.rs`)

Spawns a background thread that:
1. Creates a `ChannelProgress` (implements `ProgressHandler`)
2. Calls `duplff_core::find_duplicates()`
3. Sends `ScanMessage` variants over a channel: `ScanProgress`, `HashProgress`, `Complete`, `Error`

The TUI main loop polls this channel on each tick.

### Rendering (`mod.rs`)

Uses ratatui with crossterm backend. Main loop:
1. Poll crossterm events (key presses)
2. Poll scan channel messages
3. Render current state
4. Repeat until quit

Layout: horizontal split — groups pane (left) and detail pane (right).

### Key Bindings

| Key | State | Action |
|-----|-------|--------|
| `q` / `Esc` | Any | Quit or go back |
| `j`/`k`/arrows | Results | Navigate lists |
| `Enter` | Groups | Open group in detail pane |
| `Space` | Detail | Toggle file selection |
| `D` | Detail | Mark all duplicates in group |
| `u` | Detail | Unmark all in group |
| `d` | Results | Confirm trash selected |
| `y` | Confirm | Execute trash |
| `n` | Confirm | Cancel |
| `s` | Results | Cycle sort mode |
| `/` | Results | Enter filter mode |
| `?` | Results | Toggle help overlay |
| `Tab` | Results | Switch focus pane |
| `o` | Detail | Open file in file manager |

## Non-Interactive Modes (`non_interactive.rs`)

- `run_json()` — Pretty-printed JSON of DuplicateReport to stdout
- `run_csv()` — CSV with columns: Group, Hash, Path, Size, Status, Reason
- `run_dry_run()` — Human-readable plan: files to delete, bytes to reclaim

All use `NoopProgress` since there's no UI to update.

## Format Utilities (`format.rs`)

- `human_bytes(u64) -> String` — via humansize crate (e.g., "4.2 MB")
- `truncate_path(&str, max_width) -> String` — ellipsize from left if too long
