// Progress reporting for duplff-core

/// Trait for receiving progress updates during scanning and hashing.
///
/// Implement this trait to integrate with a UI (TUI or GUI).
/// Use `NoopProgress` for headless or test scenarios.
pub trait ProgressHandler: Send + Sync {
    /// Called periodically during directory scanning.
    fn on_scan_progress(&self, files_found: usize);

    /// Called periodically during hashing.
    fn on_hash_progress(&self, files_hashed: usize, total: usize);

    /// Called when the full pipeline completes.
    fn on_complete(&self, groups_found: usize);
}

/// A no-op progress handler for tests and non-interactive use.
pub struct NoopProgress;

impl ProgressHandler for NoopProgress {
    fn on_scan_progress(&self, _files_found: usize) {}
    fn on_hash_progress(&self, _files_hashed: usize, _total: usize) {}
    fn on_complete(&self, _groups_found: usize) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_progress_does_not_panic() {
        let p = NoopProgress;
        p.on_scan_progress(100);
        p.on_hash_progress(50, 100);
        p.on_complete(10);
    }
}
