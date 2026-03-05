# duplff-cli

Terminal interface for duplff. Supports both an interactive TUI and non-interactive output modes.

## Interactive TUI

Run with just paths to launch the TUI:

```
duplff ~/Documents ~/Downloads
```

The TUI has three views:

- **Scan** — Real-time progress showing files found, hashing progress, and elapsed time.
- **Results** — Table of duplicate groups sorted by wasted space. Press `/` to search, `s` to cycle sort mode, `a` to auto-select all duplicates, `d` to trash selected.
- **Detail** — View all files in a group. Space to toggle selection, `d` to trash, `u` to undo, `o` to open in file manager.

### Key Bindings

| Key | Action |
|-----|--------|
| `q` / `Esc` | Quit / Back |
| `j` / `k` / `↑` / `↓` | Navigate |
| `Enter` | Open group detail |
| `Space` | Toggle file selection |
| `a` | Auto-select all duplicates |
| `d` | Trash selected files |
| `u` | Undo last trash |
| `s` | Cycle sort mode |
| `/` | Search / filter |
| `o` | Open in file manager |

## Non-Interactive Modes

```
duplff --json ~/Documents         # JSON report to stdout
duplff --csv ~/Documents          # CSV report to stdout
duplff --dry-run ~/Documents      # Show what would be deleted
```

## Common Options

```
-e py rs js          # Only scan these extensions
-m 1024              # Min file size (bytes)
-M 10485760          # Max file size (bytes)
-p ~/important       # Priority directory (prefer keeping files here)
-x node_modules      # Exclude pattern (repeatable)
-L                   # Follow symlinks
--paranoid           # Byte-by-byte verification
--no-cache           # Disable hash cache
```
