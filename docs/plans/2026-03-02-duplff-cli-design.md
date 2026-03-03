# duplff-cli Design Document

**Date:** 2026-03-02
**Status:** Approved
**Scope:** CLI + TUI frontend crate

## Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| CLI scope | Both interactive TUI + non-interactive (--json, --dry-run) | Cheap to add non-interactive since core does heavy lifting |
| Arg parser | clap (derive) | Industry standard, auto-generated help |
| Threading | std::sync::mpsc channels | Simple, no async runtime, ratatui-friendly |
| TUI layout | Top groups / Bottom detail | Matches requirements, efficient for terminal |
| Architecture | Elm-style state machine | Clean separation, testable, extensible |

## Crate Structure

```
crates/duplff-cli/
├── Cargo.toml
└── src/
    ├── main.rs              # Entry point, arg parsing, mode dispatch
    ├── cli.rs               # Clap argument definitions
    ├── non_interactive.rs   # --json / --dry-run non-interactive mode
    ├── tui/
    │   ├── mod.rs           # TUI app struct, main event loop
    │   ├── state.rs         # AppState enum (Scanning, Results, Confirm, Help)
    │   ├── scan.rs          # Background scan thread + channel messages
    │   ├── groups.rs        # Groups list pane (top)
    │   ├── detail.rs        # Group detail pane (bottom)
    │   └── help.rs          # Help overlay
    └── format.rs            # Human-readable size formatting, path truncation
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `duplff-core` | Core library (path dep) |
| `clap` (derive) | Argument parsing |
| `ratatui` | TUI framework |
| `crossterm` | Terminal backend |
| `humansize` | Human-readable file sizes |

## CLI Interface

```
duplff <PATHS>... [OPTIONS]

Arguments:
  <PATHS>...              Directories to scan for duplicates

Options:
  -e, --ext <EXT>...      File extensions to include (e.g. py rs js)
  -m, --min-size <BYTES>  Minimum file size (default: 1)
  -M, --max-size <BYTES>  Maximum file size
  -p, --priority <DIR>... Priority directories (files here preferred to keep)
  -L, --follow-symlinks   Follow symbolic links
      --json              Output JSON report (non-interactive)
      --dry-run           Show deletion plan without deleting (non-interactive)
  -h, --help              Print help
  -V, --version           Print version
```

## Mode Dispatch

- `--json` -> scan, print DuplicateReport as JSON to stdout, exit
- `--dry-run` -> scan, print human-readable deletion plan, exit
- Default -> launch interactive TUI

## TUI State Machine

```
         +-----------+
  start-->  Scanning |
         +-----+-----+
               | Complete(report)
         +-----v-----+
         |  Results   |<-----------+
         +--+--+------+            |
            |  | '?'          dismiss
            |  +---> Help ---------+
            | 'd' on marked
         +--v--------+
         |  Confirm   |--yes--> trash, back to Results
         +-----------+---no---> back to Results
```

**States:**
- `Scanning { files_found, files_hashed, total_to_hash }` -- progress display
- `Results { report, group_cursor, detail_cursor, marked: HashSet<PathBuf> }` -- main interaction
- `Confirm { message }` -- "Delete N files? y/n"
- `Help` -- keybinding reference overlay

## TUI Layout

```
+--[ Duplicate Groups ]-------------------+
| # | Files | Size   | Sample Path        |
|---|-------|--------|--------------------|
| 1 |   3   | 10.0KB | /src/main.rs       |
|>2 |   2   | 4.5KB  | /lib/utils.py      |
| 3 |   2   | 1.2KB  | /test/helper.js    |
+-----------------------------------------+
+--[ Group Detail ]----------[ 2 of 3 ]---+
| [KEEP] /lib/utils.py                    |
|        Reason: deepest path             |
| [DEL]  /tmp/utils.py                    |
|                                         |
+-----------------------------------------+
  q:quit  d:delete  k:keep  Space:toggle
  D:mark-all  ?:help
```

## Keybindings

| Key | Context | Action |
|-----|---------|--------|
| q / Esc | Any | Quit (Results) or dismiss (Help/Confirm) |
| j / Down | Results | Move cursor down |
| k / Up | Results | Move cursor up |
| Tab | Results | Switch focus between groups and detail pane |
| Enter | Groups | Select group, focus detail pane |
| Space | Detail | Toggle file marked for deletion |
| d | Results | Trash marked files (shows Confirm first) |
| D | Detail | Mark all duplicates in current group |
| u | Detail | Unmark all in current group |
| ? | Any | Show help overlay |

## Colors

- Keep files: green
- Marked for deletion: red
- Selected group row: bold/highlighted
- Help text: dim

## Non-Interactive Output

**--json:** Pretty-printed DuplicateReport JSON to stdout.

**--dry-run:** Human-readable plan:
```
Found 5 duplicate groups (12 files, 45.2 MB wasted)

Group 1 (3 files, 10.0 KB each):
  [KEEP] /src/main.rs (deepest path)
  [DEL]  /backup/main.rs
  [DEL]  /tmp/main.rs
...
Total: 8 files to delete, 35.2 MB to reclaim
```

## Threading Model

- Main thread: runs ratatui event loop (crossterm events + channel polling)
- Scan thread: spawned with std::thread::spawn, sends ScanMessage via mpsc channel
- TUI polls channel with try_recv() on each tick (~60ms)

```rust
enum ScanMessage {
    ScanProgress(usize),
    HashProgress(usize, usize),
    Complete(DuplicateReport),
    Error(DuplffError),
}
```
