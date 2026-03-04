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
        filter: Option<String>,
    },
    /// Confirmation dialog before deletion.
    Confirm {
        report: DuplicateReport,
        group_cursor: usize,
        detail_cursor: usize,
        focus: FocusPane,
        marked: HashSet<PathBuf>,
        filter: Option<String>,
        message: String,
    },
    /// Help overlay.
    Help {
        report: DuplicateReport,
        group_cursor: usize,
        detail_cursor: usize,
        focus: FocusPane,
        marked: HashSet<PathBuf>,
        filter: Option<String>,
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
            filter: None,
        }
    }
}
