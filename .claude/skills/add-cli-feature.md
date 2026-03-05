# Skill: Add a Feature to duplff-cli

Use when adding new CLI arguments, TUI views, or non-interactive output modes.

## Context

Read `.claude/docs/cli-guide.md` for the TUI architecture and key bindings.

**IMPORTANT**: Do NOT modify duplff-core files unless the task explicitly requires core changes.

## Adding a CLI Argument

1. Add the field to the `Cli` struct in `crates/duplff-cli/src/cli.rs` using clap derive attributes
2. Map it to `ScanConfig` in `main.rs` where the config is built
3. If it's a new non-interactive mode, add a `run_{mode}()` function in `non_interactive.rs` and dispatch from `main.rs`

## Adding a TUI Feature

1. **State** — Add any new state fields to the relevant `AppState` variant in `tui/state.rs`
2. **Input** — Handle new key bindings in the main input match in `tui/mod.rs`
3. **Rendering** — Update the relevant render function in `tui/groups.rs`, `tui/detail.rs`, or `tui/help.rs`
4. **Help** — If adding key bindings, update the help text in `tui/help.rs`

## TUI Rendering Pattern

All rendering uses ratatui widgets:
- `Block` with `Borders` for sections
- `Paragraph` for text
- `Table` with `Row` for lists
- `Layout` with `Constraint` for splitting areas
- Colors from ratatui's `Color` enum (green for keep, red for delete, yellow for warnings)

## Verify

```bash
export PATH="$HOME/.cargo/bin:$PATH"
cargo test -p duplff-cli
cargo clippy -p duplff-cli -- -D warnings
cargo run -p duplff-cli -- --help  # verify new arg shows up
```

## Conventions

- Binary name is `duplff` (set in Cargo.toml)
- Use `humansize` crate for byte formatting (via `format::human_bytes`)
- Key bindings follow vim conventions (j/k for navigation)
- Non-interactive modes print to stdout and exit — no TUI
