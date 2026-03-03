use duplff_core::models::{DuplicateReport, ScanConfig};
use duplff_core::progress::ProgressHandler;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::Sender;

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
