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
