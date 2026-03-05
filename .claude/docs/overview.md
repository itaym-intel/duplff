# duplff — Project Overview

duplff is a duplicate file finder for Linux, written in Rust. It scans directories, identifies duplicate files via BLAKE3 hashing, ranks which copy to keep, and safely moves duplicates to the OS trash.

## Workspace Structure

```
duplff/
├── crates/
│   ├── duplff-core/       # Library — all detection and action logic
│   ├── duplff-cli/        # Binary — TUI + non-interactive modes (ratatui, clap)
│   └── duplff-gui/        # Desktop app — Tauri v2 + SvelteKit + Tailwind CSS v4
│       ├── src-tauri/     # Rust backend (Tauri commands, state)
│       └── src/           # Svelte frontend (screens, components, stores)
├── docs/                  # User-facing documentation
├── .github/workflows/     # CI (test+build) and Release (version tags)
└── .claude/               # Claude Code docs and skills
```

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Core | Rust, BLAKE3, ignore, rayon, trash, rusqlite |
| CLI | clap (args), ratatui + crossterm (TUI) |
| GUI backend | Tauri v2, serde |
| GUI frontend | SvelteKit, Svelte 5, Tailwind CSS v4 |
| CI/CD | GitHub Actions, softprops/action-gh-release |

## How It Works

1. **Scan** — Walk directories with the `ignore` crate (respects .gitignore). Filter by extension, size, exclude patterns.
2. **Group by size** — Files with unique sizes are discarded (can't be duplicates).
3. **Partial hash** — BLAKE3 of first 4KB. Unique partial hashes are discarded.
4. **Full hash** — BLAKE3 of entire file. Groups by matching full hash.
5. **Rank** — Pick the "keep" file per group: priority path > deepest path > newest mtime > lexicographic first.
6. **Report** — Return `DuplicateReport` with groups and aggregate stats.

## Data Flow

```
User config → find_duplicates() → DuplicateReport
                                       ↓
                              dry_run() or trash_duplicates()
                                       ↓
                              ActionLog → save to ~/.local/share/duplff/logs/
                                       ↓
                              undo() → restore from OS trash
```

## Key Paths on Disk

| Path | Purpose |
|------|---------|
| `~/.cache/duplff/hashes.db` | SQLite hash cache (partial + full hashes) |
| `~/.local/share/duplff/logs/*.json` | Action logs for undo support |

## Version

- Rust 1.93.1, Node 20
- All crates at version 0.1.0
- 51 tests (41 core + 4 CLI + 6 integration)

## User Preferences (MUST follow)

- **NEVER modify duplff-core files** unless the task explicitly requires core changes
- **No Co-Authored-By** on commits
- **Short commit messages**
- **Cargo commands** need: `export PATH="$HOME/.cargo/bin:$PATH"`
- **Push to main** when done (after user confirms)
- **Clean native UI** — avoid "ai slop" in GUI work
