# duplff-cli Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build the duplff CLI binary with both a non-interactive mode (--json, --dry-run) and an interactive ratatui TUI for browsing and deleting duplicate files.

**Architecture:** Elm-style state machine TUI with crossterm backend. Background scan thread communicates via std::sync::mpsc channels. Non-interactive modes share the same core API. Clap derive handles argument parsing.

**Tech Stack:** Rust, duplff-core (path dep), clap (derive), ratatui, crossterm, humansize

---

### Task 1: Scaffold duplff-cli Crate

**Files:**
- Modify: `Cargo.toml` (workspace root)
- Create: `crates/duplff-cli/Cargo.toml`
- Create: `crates/duplff-cli/src/main.rs`

**Step 1: Add duplff-cli to workspace**

In `Cargo.toml` (workspace root), change:
```toml
[workspace]
members = ["crates/duplff-core", "crates/duplff-cli"]
resolver = "2"
```

**Step 2: Create duplff-cli Cargo.toml**

```toml
[package]
name = "duplff-cli"
version = "0.1.0"
edition = "2021"
description = "CLI and TUI interface for duplff duplicate file finder"
license = "MIT"

[[bin]]
name = "duplff"
path = "src/main.rs"

[dependencies]
duplff-core = { path = "../duplff-core" }
clap = { version = "4", features = ["derive"] }
ratatui = "0.29"
crossterm = "0.28"
humansize = "2"
serde_json = "1"
```

**Step 3: Create minimal main.rs**

```rust
fn main() {
    println!("duplff - duplicate file finder");
}
```

**Step 4: Verify it compiles**

Run: `cargo build -p duplff-cli`
Expected: compiles, produces `target/debug/duplff` binary

**Step 5: Commit**

```bash
git add Cargo.toml crates/duplff-cli/
git commit -m "feat: scaffold duplff-cli crate"
```

---

### Task 2: CLI Argument Parsing

**Files:**
- Create: `crates/duplff-cli/src/cli.rs`
- Modify: `crates/duplff-cli/src/main.rs`

**Step 1: Create cli.rs with clap derive**

```rust
use clap::Parser;
use std::path::PathBuf;

/// duplff - find and remove duplicate files
#[derive(Parser, Debug)]
#[command(name = "duplff", version, about)]
pub struct Cli {
    /// Directories to scan for duplicates
    #[arg(required = true)]
    pub paths: Vec<PathBuf>,

    /// File extensions to include (e.g. py rs js)
    #[arg(short = 'e', long = "ext")]
    pub extensions: Option<Vec<String>>,

    /// Minimum file size in bytes (default: 1)
    #[arg(short = 'm', long = "min-size", default_value = "1")]
    pub min_size: u64,

    /// Maximum file size in bytes
    #[arg(short = 'M', long = "max-size")]
    pub max_size: Option<u64>,

    /// Priority directories (files here are preferred to keep)
    #[arg(short = 'p', long = "priority")]
    pub priority: Option<Vec<PathBuf>>,

    /// Follow symbolic links
    #[arg(short = 'L', long = "follow-symlinks")]
    pub follow_symlinks: bool,

    /// Output JSON report (non-interactive)
    #[arg(long)]
    pub json: bool,

    /// Show deletion plan without deleting (non-interactive)
    #[arg(long)]
    pub dry_run: bool,
}

impl Cli {
    /// Convert CLI args into a duplff-core ScanConfig.
    pub fn to_scan_config(&self) -> duplff_core::models::ScanConfig {
        duplff_core::models::ScanConfig {
            roots: self.paths.clone(),
            extensions: self.extensions.clone(),
            min_size: self.min_size,
            max_size: self.max_size,
            priority_paths: self.priority.clone().unwrap_or_default(),
            follow_symlinks: self.follow_symlinks,
        }
    }
}
```

**Step 2: Update main.rs to parse args and dispatch**

```rust
mod cli;

use clap::Parser;
use cli::Cli;

fn main() {
    let args = Cli::parse();

    if args.json || args.dry_run {
        eprintln!("Non-interactive mode not yet implemented");
        std::process::exit(1);
    } else {
        eprintln!("TUI mode not yet implemented");
        std::process::exit(1);
    }
}
```

**Step 3: Verify it compiles and --help works**

Run: `cargo run -p duplff-cli -- --help`
Expected: prints help text with all options

**Step 4: Commit**

```bash
git add crates/duplff-cli/src/cli.rs crates/duplff-cli/src/main.rs
git commit -m "feat: add CLI argument parsing with clap"
```

---

### Task 3: Format Utilities

**Files:**
- Create: `crates/duplff-cli/src/format.rs`

**Step 1: Write the implementation**

```rust
use humansize::{format_size, BINARY};

/// Format a byte count as a human-readable string (e.g. "14.3 KiB").
pub fn human_bytes(bytes: u64) -> String {
    format_size(bytes, BINARY)
}

/// Truncate a path string to fit within max_width, keeping the end visible.
///
/// If the path is longer than max_width, replaces the beginning with "...".
pub fn truncate_path(path: &str, max_width: usize) -> String {
    if path.len() <= max_width {
        return path.to_string();
    }
    if max_width <= 3 {
        return "...".to_string();
    }
    format!("...{}", &path[path.len() - (max_width - 3)..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn human_bytes_formats_correctly() {
        assert_eq!(human_bytes(0), "0 B");
        assert_eq!(human_bytes(1024), "1 KiB");
        assert_eq!(human_bytes(1048576), "1 MiB");
    }

    #[test]
    fn truncate_path_short_path_unchanged() {
        assert_eq!(truncate_path("/a/b.txt", 20), "/a/b.txt");
    }

    #[test]
    fn truncate_path_long_path_truncated() {
        let long = "/very/long/path/to/some/deeply/nested/file.txt";
        let result = truncate_path(long, 20);
        assert!(result.starts_with("..."));
        assert_eq!(result.len(), 20);
    }

    #[test]
    fn truncate_path_tiny_width() {
        assert_eq!(truncate_path("/a/b/c/d.txt", 3), "...");
    }
}
```

**Step 2: Add module to main.rs**

Add `mod format;` to main.rs.

**Step 3: Run tests**

Run: `cargo test -p duplff-cli`
Expected: 4 tests PASS

**Step 4: Commit**

```bash
git add crates/duplff-cli/src/format.rs crates/duplff-cli/src/main.rs
git commit -m "feat: add format utilities for human-readable sizes and path truncation"
```

---

### Task 4: Non-Interactive Mode (--json and --dry-run)

**Files:**
- Create: `crates/duplff-cli/src/non_interactive.rs`
- Modify: `crates/duplff-cli/src/main.rs`

**Step 1: Implement non_interactive.rs**

```rust
use crate::format::human_bytes;
use duplff_core::actions;
use duplff_core::models::{DuplicateReport, ScanConfig};
use duplff_core::progress::NoopProgress;

/// Run in JSON mode: scan and print the report as JSON to stdout.
pub fn run_json(config: &ScanConfig) -> Result<(), Box<dyn std::error::Error>> {
    let report = duplff_core::find_duplicates(config, &NoopProgress)?;
    let json = serde_json::to_string_pretty(&report)?;
    println!("{json}");
    Ok(())
}

/// Run in dry-run mode: scan and print a human-readable deletion plan.
pub fn run_dry_run(config: &ScanConfig) -> Result<(), Box<dyn std::error::Error>> {
    let report = duplff_core::find_duplicates(config, &NoopProgress)?;

    if report.groups.is_empty() {
        println!("No duplicates found.");
        return Ok(());
    }

    println!(
        "Found {} duplicate group{} ({} files, {} wasted)\n",
        report.groups.len(),
        if report.groups.len() == 1 { "" } else { "s" },
        report.total_files_scanned,
        human_bytes(report.total_wasted_bytes),
    );

    for (i, group) in report.groups.iter().enumerate() {
        let file_count = 1 + group.duplicates.len();
        println!(
            "Group {} ({} files, {} each):",
            i + 1,
            file_count,
            human_bytes(group.size),
        );
        println!(
            "  [KEEP] {} ({})",
            group.keep.entry.path.display(),
            group.keep.reason,
        );
        for dup in &group.duplicates {
            println!("  [DEL]  {}", dup.entry.path.display());
        }
        println!();
    }

    let plan = actions::dry_run(&report.groups);
    println!(
        "Total: {} file{} to delete, {} to reclaim",
        plan.files_to_delete.len(),
        if plan.files_to_delete.len() == 1 { "" } else { "s" },
        human_bytes(plan.bytes_to_reclaim),
    );

    Ok(())
}
```

**Step 2: Update main.rs to wire up non-interactive modes**

```rust
mod cli;
mod format;
mod non_interactive;

use clap::Parser;
use cli::Cli;

fn main() {
    let args = Cli::parse();
    let config = args.to_scan_config();

    let result = if args.json {
        non_interactive::run_json(&config)
    } else if args.dry_run {
        non_interactive::run_dry_run(&config)
    } else {
        eprintln!("TUI mode not yet implemented");
        std::process::exit(1);
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
```

**Step 3: Test --json mode manually**

Create a test directory and run:
```bash
mkdir -p /tmp/duplff-test && echo "hello" > /tmp/duplff-test/a.txt && echo "hello" > /tmp/duplff-test/b.txt
cargo run -p duplff-cli -- /tmp/duplff-test --json
```
Expected: JSON output with 1 duplicate group

**Step 4: Test --dry-run mode manually**

```bash
cargo run -p duplff-cli -- /tmp/duplff-test --dry-run
```
Expected: Human-readable output showing group with [KEEP] and [DEL]

**Step 5: Commit**

```bash
git add crates/duplff-cli/src/non_interactive.rs crates/duplff-cli/src/main.rs
git commit -m "feat: add non-interactive --json and --dry-run modes"
```

---

### Task 5: TUI Scan Thread and Messages

**Files:**
- Create: `crates/duplff-cli/src/tui/mod.rs`
- Create: `crates/duplff-cli/src/tui/scan.rs`

**Step 1: Create tui/scan.rs — background scan with channel messages**

```rust
use duplff_core::error::DuplffError;
use duplff_core::models::{DuplicateReport, ScanConfig};
use duplff_core::progress::ProgressHandler;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;

/// Messages sent from the scan thread to the TUI.
pub enum ScanMessage {
    ScanProgress(usize),
    HashProgress { done: usize, total: usize },
    Complete(DuplicateReport),
    Error(String),
}

/// A ProgressHandler that sends messages through an mpsc channel.
struct ChannelProgress {
    tx: Sender<ScanMessage>,
    scan_count: AtomicUsize,
    hash_count: AtomicUsize,
}

impl ProgressHandler for ChannelProgress {
    fn on_scan_progress(&self, files_found: usize) {
        let prev = self.scan_count.swap(files_found, Ordering::Relaxed);
        // Only send if changed significantly (every 500 files) to avoid flooding
        if files_found / 500 != prev / 500 || files_found == 0 {
            let _ = self.tx.send(ScanMessage::ScanProgress(files_found));
        }
    }

    fn on_hash_progress(&self, files_hashed: usize, total: usize) {
        let prev = self.hash_count.swap(files_hashed, Ordering::Relaxed);
        if files_hashed / 50 != prev / 50 {
            let _ = self.tx.send(ScanMessage::HashProgress {
                done: files_hashed,
                total,
            });
        }
    }

    fn on_complete(&self, _groups_found: usize) {
        // Complete message is sent after find_duplicates returns
    }
}

/// Spawn a background thread that runs the scan and sends messages via the channel.
pub fn spawn_scan(config: ScanConfig, tx: Sender<ScanMessage>) {
    std::thread::spawn(move || {
        let progress = ChannelProgress {
            tx: tx.clone(),
            scan_count: AtomicUsize::new(0),
            hash_count: AtomicUsize::new(0),
        };

        match duplff_core::find_duplicates(&config, &progress) {
            Ok(report) => {
                let _ = tx.send(ScanMessage::Complete(report));
            }
            Err(e) => {
                let _ = tx.send(ScanMessage::Error(e.to_string()));
            }
        }
    });
}
```

**Step 2: Create tui/mod.rs — placeholder module**

```rust
pub mod scan;
```

**Step 3: Add `mod tui;` to main.rs**

Add the module declaration to main.rs (after the other mod lines).

**Step 4: Verify it compiles**

Run: `cargo build -p duplff-cli`
Expected: compiles

**Step 5: Commit**

```bash
git add crates/duplff-cli/src/tui/
git commit -m "feat: add background scan thread with channel-based progress"
```

---

### Task 6: TUI App State

**Files:**
- Create: `crates/duplff-cli/src/tui/state.rs`
- Modify: `crates/duplff-cli/src/tui/mod.rs`

**Step 1: Implement state.rs**

```rust
use duplff_core::models::DuplicateReport;
use std::collections::HashSet;
use std::path::PathBuf;

/// Which pane has focus in Results view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPane {
    Groups,
    Detail,
}

/// The application state machine.
pub enum AppState {
    /// Scanning in progress.
    Scanning {
        files_found: usize,
        files_hashed: usize,
        total_to_hash: usize,
        phase: ScanPhase,
    },
    /// Scan complete, showing results.
    Results {
        report: DuplicateReport,
        group_cursor: usize,
        detail_cursor: usize,
        focus: FocusPane,
        marked: HashSet<PathBuf>,
    },
    /// Confirmation dialog before deletion.
    Confirm {
        report: DuplicateReport,
        group_cursor: usize,
        detail_cursor: usize,
        focus: FocusPane,
        marked: HashSet<PathBuf>,
        message: String,
    },
    /// Help overlay.
    Help {
        report: DuplicateReport,
        group_cursor: usize,
        detail_cursor: usize,
        focus: FocusPane,
        marked: HashSet<PathBuf>,
    },
    /// Fatal error.
    Error(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanPhase {
    Scanning,
    Hashing,
}

impl AppState {
    /// Create the initial scanning state.
    pub fn scanning() -> Self {
        AppState::Scanning {
            files_found: 0,
            files_hashed: 0,
            total_to_hash: 0,
            phase: ScanPhase::Scanning,
        }
    }

    /// Transition from scan complete to results view.
    pub fn into_results(report: DuplicateReport) -> Self {
        AppState::Results {
            report,
            group_cursor: 0,
            detail_cursor: 0,
            focus: FocusPane::Groups,
            marked: HashSet::new(),
        }
    }
}
```

**Step 2: Update tui/mod.rs**

```rust
pub mod scan;
pub mod state;
```

**Step 3: Verify it compiles**

Run: `cargo build -p duplff-cli`
Expected: compiles

**Step 4: Commit**

```bash
git add crates/duplff-cli/src/tui/state.rs crates/duplff-cli/src/tui/mod.rs
git commit -m "feat: add TUI app state machine"
```

---

### Task 7: TUI Help Pane

**Files:**
- Create: `crates/duplff-cli/src/tui/help.rs`
- Modify: `crates/duplff-cli/src/tui/mod.rs`

**Step 1: Implement help.rs**

```rust
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

/// Render the help overlay centered in the given area.
pub fn render_help(frame: &mut Frame, area: Rect) {
    let help_lines = vec![
        Line::from(vec![
            Span::styled("Key", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("          "),
            Span::styled("Action", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("j / Down    ", Style::default().fg(Color::Yellow)),
            Span::raw("Move cursor down"),
        ]),
        Line::from(vec![
            Span::styled("k / Up      ", Style::default().fg(Color::Yellow)),
            Span::raw("Move cursor up"),
        ]),
        Line::from(vec![
            Span::styled("Tab         ", Style::default().fg(Color::Yellow)),
            Span::raw("Switch focus (groups / detail)"),
        ]),
        Line::from(vec![
            Span::styled("Enter       ", Style::default().fg(Color::Yellow)),
            Span::raw("Select group, focus detail"),
        ]),
        Line::from(vec![
            Span::styled("Space       ", Style::default().fg(Color::Yellow)),
            Span::raw("Toggle file for deletion"),
        ]),
        Line::from(vec![
            Span::styled("D           ", Style::default().fg(Color::Yellow)),
            Span::raw("Mark all duplicates in group"),
        ]),
        Line::from(vec![
            Span::styled("u           ", Style::default().fg(Color::Yellow)),
            Span::raw("Unmark all in group"),
        ]),
        Line::from(vec![
            Span::styled("d           ", Style::default().fg(Color::Yellow)),
            Span::raw("Delete marked files (to trash)"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("q / Esc     ", Style::default().fg(Color::Yellow)),
            Span::raw("Quit / dismiss"),
        ]),
        Line::from(vec![
            Span::styled("?           ", Style::default().fg(Color::Yellow)),
            Span::raw("Toggle this help"),
        ]),
    ];

    // Center a box in the terminal
    let width = 50.min(area.width.saturating_sub(4));
    let height = (help_lines.len() as u16 + 2).min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    let help_area = Rect::new(x, y, width, height);

    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White).bg(Color::DarkGray));

    let paragraph = Paragraph::new(help_lines).block(block);

    frame.render_widget(Clear, help_area);
    frame.render_widget(paragraph, help_area);
}
```

**Step 2: Update tui/mod.rs**

```rust
pub mod help;
pub mod scan;
pub mod state;
```

**Step 3: Verify it compiles**

Run: `cargo build -p duplff-cli`
Expected: compiles

**Step 4: Commit**

```bash
git add crates/duplff-cli/src/tui/help.rs crates/duplff-cli/src/tui/mod.rs
git commit -m "feat: add TUI help overlay"
```

---

### Task 8: Groups Pane Rendering

**Files:**
- Create: `crates/duplff-cli/src/tui/groups.rs`
- Modify: `crates/duplff-cli/src/tui/mod.rs`

**Step 1: Implement groups.rs**

```rust
use crate::format::human_bytes;
use duplff_core::models::DuplicateGroup;
use ratatui::layout::Constraint;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table, TableState};
use ratatui::Frame;

/// Render the duplicate groups table in the given area.
pub fn render_groups(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    groups: &[DuplicateGroup],
    cursor: usize,
    focused: bool,
) {
    let header = Row::new(vec![
        Cell::from("#"),
        Cell::from("Files"),
        Cell::from("Size"),
        Cell::from("Wasted"),
        Cell::from("Sample Path"),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD))
    .height(1);

    let rows: Vec<Row> = groups
        .iter()
        .enumerate()
        .map(|(i, group)| {
            let file_count = 1 + group.duplicates.len();
            Row::new(vec![
                Cell::from(format!("{}", i + 1)),
                Cell::from(format!("{}", file_count)),
                Cell::from(human_bytes(group.size)),
                Cell::from(human_bytes(group.wasted_bytes())),
                Cell::from(group.keep.entry.path.display().to_string()),
            ])
        })
        .collect();

    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(4),
            Constraint::Length(6),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Fill(1),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(" Duplicate Groups ")
            .borders(Borders::ALL)
            .border_style(border_style),
    )
    .row_highlight_style(
        Style::default()
            .add_modifier(Modifier::BOLD)
            .bg(Color::DarkGray),
    );

    let mut table_state = TableState::default();
    if !groups.is_empty() {
        table_state.select(Some(cursor));
    }
    frame.render_stateful_widget(table, area, &mut table_state);
}
```

**Step 2: Update tui/mod.rs**

```rust
pub mod groups;
pub mod help;
pub mod scan;
pub mod state;
```

**Step 3: Verify it compiles**

Run: `cargo build -p duplff-cli`
Expected: compiles

**Step 4: Commit**

```bash
git add crates/duplff-cli/src/tui/groups.rs crates/duplff-cli/src/tui/mod.rs
git commit -m "feat: add groups pane rendering"
```

---

### Task 9: Detail Pane Rendering

**Files:**
- Create: `crates/duplff-cli/src/tui/detail.rs`
- Modify: `crates/duplff-cli/src/tui/mod.rs`

**Step 1: Implement detail.rs**

```rust
use duplff_core::models::DuplicateGroup;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::Frame;
use std::collections::HashSet;
use std::path::PathBuf;

/// Render the detail pane showing files in the selected group.
pub fn render_detail(
    frame: &mut Frame,
    area: Rect,
    group: Option<&DuplicateGroup>,
    cursor: usize,
    focused: bool,
    marked: &HashSet<PathBuf>,
) {
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let Some(group) = group else {
        let block = Block::default()
            .title(" Group Detail ")
            .borders(Borders::ALL)
            .border_style(border_style);
        frame.render_widget(block, area);
        return;
    };

    let total_files = 1 + group.duplicates.len();
    let title = format!(" Group Detail [{} files] ", total_files);

    // Build list items: keep file first, then duplicates
    let mut items: Vec<ListItem> = Vec::with_capacity(total_files);

    // Keep file
    let keep_path = group.keep.entry.path.display().to_string();
    let keep_line = Line::from(vec![
        Span::styled("[KEEP] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw(&keep_path),
    ]);
    let reason_line = Line::from(vec![
        Span::raw("       Reason: "),
        Span::styled(
            group.keep.reason.to_string(),
            Style::default().fg(Color::Green),
        ),
    ]);
    items.push(ListItem::new(vec![keep_line, reason_line]));

    // Duplicate files
    for dup in &group.duplicates {
        let is_marked = marked.contains(&dup.entry.path);
        let dup_path = dup.entry.path.display().to_string();

        let (tag, style) = if is_marked {
            ("[DEL]  ", Style::default().fg(Color::Red))
        } else {
            ("[   ]  ", Style::default())
        };

        let line = Line::from(vec![
            Span::styled(tag, style.add_modifier(Modifier::BOLD)),
            Span::styled(&dup_path, style),
        ]);
        items.push(ListItem::new(vec![line]));
    }

    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::DarkGray),
        );

    let mut list_state = ListState::default();
    list_state.select(Some(cursor));
    frame.render_stateful_widget(list, area, &mut list_state);
}
```

**Step 2: Update tui/mod.rs**

```rust
pub mod detail;
pub mod groups;
pub mod help;
pub mod scan;
pub mod state;
```

**Step 3: Verify it compiles**

Run: `cargo build -p duplff-cli`
Expected: compiles

**Step 4: Commit**

```bash
git add crates/duplff-cli/src/tui/detail.rs crates/duplff-cli/src/tui/mod.rs
git commit -m "feat: add detail pane rendering with keep/delete markers"
```

---

### Task 10: TUI Main Event Loop

**Files:**
- Modify: `crates/duplff-cli/src/tui/mod.rs` (major rewrite)
- Modify: `crates/duplff-cli/src/main.rs`

This is the largest task — it wires everything together: terminal init, event loop, state transitions, rendering, and input handling.

**Step 1: Implement the full tui/mod.rs event loop**

```rust
pub mod detail;
pub mod groups;
pub mod help;
pub mod scan;
pub mod state;

use crate::format::human_bytes;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use duplff_core::actions;
use duplff_core::models::ScanConfig;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use state::{AppState, FocusPane, ScanPhase};
use std::io::stdout;
use std::sync::mpsc;
use std::time::Duration;

/// Run the interactive TUI.
pub fn run(config: ScanConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = ratatui::init();

    // Start background scan
    let (tx, rx) = mpsc::channel();
    scan::spawn_scan(config, tx);

    let mut state = AppState::scanning();
    let mut should_quit = false;

    while !should_quit {
        // Poll for scan messages
        while let Ok(msg) = rx.try_recv() {
            match msg {
                scan::ScanMessage::ScanProgress(count) => {
                    if let AppState::Scanning {
                        ref mut files_found,
                        ref mut phase,
                        ..
                    } = state
                    {
                        *files_found = count;
                        *phase = ScanPhase::Scanning;
                    }
                }
                scan::ScanMessage::HashProgress { done, total } => {
                    if let AppState::Scanning {
                        ref mut files_hashed,
                        ref mut total_to_hash,
                        ref mut phase,
                        ..
                    } = state
                    {
                        *files_hashed = done;
                        *total_to_hash = total;
                        *phase = ScanPhase::Hashing;
                    }
                }
                scan::ScanMessage::Complete(report) => {
                    state = AppState::into_results(report);
                }
                scan::ScanMessage::Error(msg) => {
                    state = AppState::Error(msg);
                }
            }
        }

        // Render
        terminal.draw(|frame| render(frame, &state))?;

        // Handle input (poll with timeout so we keep processing scan messages)
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                should_quit = handle_input(key, &mut state);
            }
        }
    }

    // Cleanup terminal
    ratatui::restore();
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}

/// Render the current state to the frame.
fn render(frame: &mut Frame, state: &AppState) {
    let area = frame.area();

    match state {
        AppState::Scanning {
            files_found,
            files_hashed,
            total_to_hash,
            phase,
        } => {
            render_scanning(frame, area, *phase, *files_found, *files_hashed, *total_to_hash);
        }
        AppState::Results {
            report,
            group_cursor,
            detail_cursor,
            focus,
            marked,
        } => {
            render_results(frame, area, report, *group_cursor, *detail_cursor, *focus, marked);
        }
        AppState::Confirm {
            report,
            group_cursor,
            detail_cursor,
            focus,
            marked,
            message,
        } => {
            render_results(frame, area, report, *group_cursor, *detail_cursor, *focus, marked);
            render_confirm(frame, area, message);
        }
        AppState::Help {
            report,
            group_cursor,
            detail_cursor,
            focus,
            marked,
        } => {
            render_results(frame, area, report, *group_cursor, *detail_cursor, *focus, marked);
            help::render_help(frame, area);
        }
        AppState::Error(msg) => {
            let paragraph = Paragraph::new(format!("Error: {msg}\n\nPress q to quit."))
                .style(Style::default().fg(Color::Red));
            frame.render_widget(paragraph, area);
        }
    }
}

fn render_scanning(
    frame: &mut Frame,
    area: Rect,
    phase: ScanPhase,
    files_found: usize,
    files_hashed: usize,
    total_to_hash: usize,
) {
    let text = match phase {
        ScanPhase::Scanning => format!("Scanning... {} files found", files_found),
        ScanPhase::Hashing => format!("Hashing... {} / {} files", files_hashed, total_to_hash),
    };
    let paragraph = Paragraph::new(text).style(Style::default().fg(Color::Cyan));
    // Center vertically
    let y = area.height / 2;
    let centered = Rect::new(area.x + 2, y, area.width.saturating_sub(4), 1);
    frame.render_widget(paragraph, centered);
}

fn render_results(
    frame: &mut Frame,
    area: Rect,
    report: &duplff_core::models::DuplicateReport,
    group_cursor: usize,
    detail_cursor: usize,
    focus: FocusPane,
    marked: &std::collections::HashSet<std::path::PathBuf>,
) {
    // Layout: summary bar (1 line) + groups pane (40%) + detail pane (rest) + help bar (1 line)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // summary
            Constraint::Percentage(40), // groups
            Constraint::Fill(1),   // detail
            Constraint::Length(1),  // help bar
        ])
        .split(area);

    // Summary bar
    let summary = Line::from(vec![
        Span::styled(
            format!(
                " {} group{}, {} duplicate{}, {} wasted ",
                report.groups.len(),
                if report.groups.len() == 1 { "" } else { "s" },
                report.total_duplicates,
                if report.total_duplicates == 1 { "" } else { "s" },
                human_bytes(report.total_wasted_bytes),
            ),
            Style::default().fg(Color::White),
        ),
        if !marked.is_empty() {
            Span::styled(
                format!(" | {} marked for deletion", marked.len()),
                Style::default().fg(Color::Red),
            )
        } else {
            Span::raw("")
        },
    ]);
    frame.render_widget(Paragraph::new(summary), chunks[0]);

    // Groups pane
    groups::render_groups(
        frame,
        chunks[1],
        &report.groups,
        group_cursor,
        focus == FocusPane::Groups,
    );

    // Detail pane
    let current_group = report.groups.get(group_cursor);
    detail::render_detail(
        frame,
        chunks[2],
        current_group,
        detail_cursor,
        focus == FocusPane::Detail,
        marked,
    );

    // Help bar
    let help_bar = Line::from(vec![
        Span::styled(" q", Style::default().fg(Color::Yellow)),
        Span::raw(":quit  "),
        Span::styled("j/k", Style::default().fg(Color::Yellow)),
        Span::raw(":nav  "),
        Span::styled("Tab", Style::default().fg(Color::Yellow)),
        Span::raw(":switch  "),
        Span::styled("Space", Style::default().fg(Color::Yellow)),
        Span::raw(":toggle  "),
        Span::styled("d", Style::default().fg(Color::Yellow)),
        Span::raw(":delete  "),
        Span::styled("D", Style::default().fg(Color::Yellow)),
        Span::raw(":mark-all  "),
        Span::styled("?", Style::default().fg(Color::Yellow)),
        Span::raw(":help"),
    ]);
    frame.render_widget(
        Paragraph::new(help_bar).style(Style::default().fg(Color::DarkGray)),
        chunks[3],
    );
}

fn render_confirm(frame: &mut Frame, area: Rect, message: &str) {
    use ratatui::widgets::{Block, Borders, Clear};

    let width = 50.min(area.width.saturating_sub(4));
    let height = 5.min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    let confirm_area = Rect::new(x, y, width, height);

    let block = Block::default()
        .title(" Confirm ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Yellow).bg(Color::DarkGray));

    let text = format!("{message}\n\nPress y to confirm, n to cancel.");
    let paragraph = Paragraph::new(text).block(block);

    frame.render_widget(Clear, confirm_area);
    frame.render_widget(paragraph, confirm_area);
}

/// Handle a key event. Returns true if the app should quit.
fn handle_input(key: KeyEvent, state: &mut AppState) -> bool {
    // Ctrl+C always quits
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return true;
    }

    match state {
        AppState::Scanning { .. } => {
            if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                return true;
            }
        }
        AppState::Error(_) => {
            if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                return true;
            }
        }
        AppState::Help {
            report,
            group_cursor,
            detail_cursor,
            focus,
            marked,
        } => {
            // Any key dismisses help
            let r = std::mem::replace(report, duplff_core::models::DuplicateReport {
                groups: vec![], total_files_scanned: 0, total_bytes_scanned: 0,
                total_duplicates: 0, total_wasted_bytes: 0,
            });
            let gc = *group_cursor;
            let dc = *detail_cursor;
            let f = *focus;
            let m = std::mem::take(marked);
            *state = AppState::Results {
                report: r,
                group_cursor: gc,
                detail_cursor: dc,
                focus: f,
                marked: m,
            };
        }
        AppState::Confirm {
            report,
            group_cursor,
            detail_cursor,
            focus,
            marked,
            ..
        } => match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                // Perform deletion
                let groups_to_delete: Vec<_> = report
                    .groups
                    .iter()
                    .map(|g| {
                        let mut g = g.clone();
                        g.duplicates.retain(|d| marked.contains(&d.entry.path));
                        g
                    })
                    .filter(|g| !g.duplicates.is_empty())
                    .collect();

                if let Err(e) = actions::trash_duplicates(&groups_to_delete) {
                    *state = AppState::Error(e.to_string());
                    return false;
                }

                // Remove trashed files from the report
                let deleted_paths: std::collections::HashSet<_> = marked.iter().cloned().collect();
                let mut new_report = std::mem::replace(report, duplff_core::models::DuplicateReport {
                    groups: vec![], total_files_scanned: 0, total_bytes_scanned: 0,
                    total_duplicates: 0, total_wasted_bytes: 0,
                });
                for group in &mut new_report.groups {
                    group.duplicates.retain(|d| !deleted_paths.contains(&d.entry.path));
                }
                new_report.groups.retain(|g| !g.duplicates.is_empty());
                new_report.total_duplicates = new_report.groups.iter().map(|g| g.duplicates.len()).sum();
                new_report.total_wasted_bytes = new_report.groups.iter().map(|g| g.wasted_bytes()).sum();

                *state = AppState::Results {
                    report: new_report,
                    group_cursor: 0,
                    detail_cursor: 0,
                    focus: FocusPane::Groups,
                    marked: std::collections::HashSet::new(),
                };
            }
            _ => {
                // Cancel — go back to results
                let r = std::mem::replace(report, duplff_core::models::DuplicateReport {
                    groups: vec![], total_files_scanned: 0, total_bytes_scanned: 0,
                    total_duplicates: 0, total_wasted_bytes: 0,
                });
                let gc = *group_cursor;
                let dc = *detail_cursor;
                let f = *focus;
                let m = std::mem::take(marked);
                *state = AppState::Results {
                    report: r,
                    group_cursor: gc,
                    detail_cursor: dc,
                    focus: f,
                    marked: m,
                };
            }
        },
        AppState::Results {
            report,
            group_cursor,
            detail_cursor,
            focus,
            marked,
        } => match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('?') => {
                let r = std::mem::replace(report, duplff_core::models::DuplicateReport {
                    groups: vec![], total_files_scanned: 0, total_bytes_scanned: 0,
                    total_duplicates: 0, total_wasted_bytes: 0,
                });
                let gc = *group_cursor;
                let dc = *detail_cursor;
                let f = *focus;
                let m = std::mem::take(marked);
                *state = AppState::Help {
                    report: r,
                    group_cursor: gc,
                    detail_cursor: dc,
                    focus: f,
                    marked: m,
                };
            }
            KeyCode::Tab => {
                *focus = match focus {
                    FocusPane::Groups => FocusPane::Detail,
                    FocusPane::Detail => FocusPane::Groups,
                };
            }
            KeyCode::Enter => {
                if *focus == FocusPane::Groups {
                    *focus = FocusPane::Detail;
                    *detail_cursor = 0;
                }
            }
            KeyCode::Char('j') | KeyCode::Down => match focus {
                FocusPane::Groups => {
                    if !report.groups.is_empty() {
                        *group_cursor = (*group_cursor + 1).min(report.groups.len() - 1);
                        *detail_cursor = 0;
                    }
                }
                FocusPane::Detail => {
                    if let Some(group) = report.groups.get(*group_cursor) {
                        let max = group.duplicates.len(); // 0 = keep, 1..=n = dups
                        *detail_cursor = (*detail_cursor + 1).min(max);
                    }
                }
            },
            KeyCode::Char('k') | KeyCode::Up => match focus {
                FocusPane::Groups => {
                    *group_cursor = group_cursor.saturating_sub(1);
                    *detail_cursor = 0;
                }
                FocusPane::Detail => {
                    *detail_cursor = detail_cursor.saturating_sub(1);
                }
            },
            KeyCode::Char(' ') => {
                if *focus == FocusPane::Detail {
                    if let Some(group) = report.groups.get(*group_cursor) {
                        // detail_cursor 0 = keep file (can't mark), 1+ = duplicates
                        if *detail_cursor > 0 {
                            let dup_idx = *detail_cursor - 1;
                            if let Some(dup) = group.duplicates.get(dup_idx) {
                                let path = dup.entry.path.clone();
                                if marked.contains(&path) {
                                    marked.remove(&path);
                                } else {
                                    marked.insert(path);
                                }
                            }
                        }
                    }
                }
            }
            KeyCode::Char('D') => {
                if *focus == FocusPane::Detail {
                    if let Some(group) = report.groups.get(*group_cursor) {
                        for dup in &group.duplicates {
                            marked.insert(dup.entry.path.clone());
                        }
                    }
                }
            }
            KeyCode::Char('u') => {
                if *focus == FocusPane::Detail {
                    if let Some(group) = report.groups.get(*group_cursor) {
                        for dup in &group.duplicates {
                            marked.remove(&dup.entry.path);
                        }
                    }
                }
            }
            KeyCode::Char('d') => {
                if !marked.is_empty() {
                    let count = marked.len();
                    let bytes: u64 = report
                        .groups
                        .iter()
                        .flat_map(|g| g.duplicates.iter())
                        .filter(|d| marked.contains(&d.entry.path))
                        .map(|d| d.entry.size)
                        .sum();
                    let message = format!(
                        "Delete {} file{} ({}) to trash?",
                        count,
                        if count == 1 { "" } else { "s" },
                        human_bytes(bytes),
                    );
                    let r = std::mem::replace(report, duplff_core::models::DuplicateReport {
                        groups: vec![], total_files_scanned: 0, total_bytes_scanned: 0,
                        total_duplicates: 0, total_wasted_bytes: 0,
                    });
                    let gc = *group_cursor;
                    let dc = *detail_cursor;
                    let f = *focus;
                    let m = std::mem::take(marked);
                    *state = AppState::Confirm {
                        report: r,
                        group_cursor: gc,
                        detail_cursor: dc,
                        focus: f,
                        marked: m,
                        message,
                    };
                }
            }
            _ => {}
        },
    }
    false
}
```

**Step 2: Update main.rs to launch TUI**

```rust
mod cli;
mod format;
mod non_interactive;
mod tui;

use clap::Parser;
use cli::Cli;

fn main() {
    let args = Cli::parse();
    let config = args.to_scan_config();

    let result = if args.json {
        non_interactive::run_json(&config)
    } else if args.dry_run {
        non_interactive::run_dry_run(&config)
    } else {
        tui::run(config)
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
```

**Step 3: Verify it compiles**

Run: `cargo build -p duplff-cli`
Expected: compiles

**Step 4: Manual smoke test**

```bash
mkdir -p /tmp/duplff-test/src && echo "hello" > /tmp/duplff-test/a.txt && echo "hello" > /tmp/duplff-test/b.txt && echo "hello" > /tmp/duplff-test/src/c.txt
cargo run -p duplff-cli -- /tmp/duplff-test
```
Expected: TUI launches, shows scanning progress, then results with 1 group of 3 files. Can navigate with j/k, Tab, toggle with Space, ? shows help, q quits.

**Step 5: Commit**

```bash
git add crates/duplff-cli/src/tui/mod.rs crates/duplff-cli/src/main.rs
git commit -m "feat: add TUI main event loop with full keyboard navigation"
```

---

### Task 11: Final Verification

**Step 1: Verify all tests pass**

Run: `cargo test --workspace`
Expected: all duplff-core tests pass, duplff-cli format tests pass

**Step 2: Run clippy**

Run: `cargo clippy --workspace -- -D warnings`
Expected: no warnings. Fix if any.

**Step 3: Run rustfmt**

Run: `cargo fmt --all -- --check`
Expected: clean. Fix if needed.

**Step 4: Full smoke test of all modes**

```bash
# Non-interactive JSON
cargo run -p duplff-cli -- /tmp/duplff-test --json

# Non-interactive dry-run
cargo run -p duplff-cli -- /tmp/duplff-test --dry-run

# Interactive TUI
cargo run -p duplff-cli -- /tmp/duplff-test
```

**Step 5: Commit any fixes**

```bash
git add -u
git commit -m "chore: fix clippy warnings and formatting"
```
