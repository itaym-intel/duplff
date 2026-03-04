//! duplff-core: Core duplicate file detection library.
//!
//! Provides scanning, hashing, grouping, ranking, and safe removal of duplicate files.

pub mod actions;
pub mod cache;
pub mod deduper;
pub mod error;
pub mod hasher;
pub mod models;
pub mod progress;
pub mod ranker;
pub mod scanner;

use error::Result;
use models::{DuplicateReport, ScanConfig};
use progress::ProgressHandler;

/// Run the full duplicate-finding pipeline.
///
/// Scans directories, groups by size, hashes candidates (partial then full),
/// ranks each group, and returns a complete report.
pub fn find_duplicates(
    config: &ScanConfig,
    progress: &dyn ProgressHandler,
) -> Result<DuplicateReport> {
    // 1. Scan directories
    let files = scanner::scan(config, progress)?;
    let total_files_scanned = files.len();
    let total_bytes_scanned: u64 = files.iter().map(|f| f.size).sum();

    // 2. Group by size, partial hash, full hash — with cache
    let cache = if config.no_cache {
        None
    } else {
        cache::HashCache::open_default().ok()
    };
    let duplicate_groups = deduper::find_duplicate_groups(files, progress, cache.as_ref())?;

    // 3. Rank groups
    let groups = ranker::rank_groups(duplicate_groups, &config.priority_paths);

    // 4. Compute stats
    let total_duplicates: usize = groups.iter().map(|g| g.duplicates.len()).sum();
    let total_wasted_bytes: u64 = groups.iter().map(|g| g.wasted_bytes()).sum();

    progress.on_complete(groups.len());

    Ok(DuplicateReport {
        groups,
        total_files_scanned,
        total_bytes_scanned,
        total_duplicates,
        total_wasted_bytes,
    })
}
