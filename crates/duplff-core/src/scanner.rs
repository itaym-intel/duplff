// File scanning for duplff-core

use crate::error::{DuplffError, Result};
use crate::models::{FileEntry, ScanConfig};
use crate::progress::ProgressHandler;
use ignore::overrides::OverrideBuilder;
use ignore::WalkBuilder;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Scan directories according to config, returning matching file entries.
pub fn scan(config: &ScanConfig, progress: &dyn ProgressHandler) -> Result<Vec<FileEntry>> {
    if config.roots.is_empty() {
        return Err(DuplffError::ScanError(
            "no root directories specified".into(),
        ));
    }

    let mut builder = WalkBuilder::new(&config.roots[0]);
    for root in &config.roots[1..] {
        builder.add(root);
    }
    builder
        .follow_links(config.follow_symlinks)
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true);

    if !config.exclude_patterns.is_empty() {
        // OverrideBuilder needs a base path for relative patterns; roots[0] is fine
        // because overrides apply to all paths once attached to WalkBuilder.
        let mut overrides = OverrideBuilder::new(&config.roots[0]);
        for pattern in &config.exclude_patterns {
            // Negate the pattern so it's excluded
            overrides
                .add(&format!("!{pattern}"))
                .map_err(|e| DuplffError::ScanError(e.to_string()))?;
        }
        let overrides = overrides
            .build()
            .map_err(|e| DuplffError::ScanError(e.to_string()))?;
        builder.overrides(overrides);
    }

    let counter = AtomicUsize::new(0);
    let mut files = Vec::new();

    for result in builder.build() {
        let entry = result.map_err(|e| DuplffError::ScanError(e.to_string()))?;

        // Skip directories
        match entry.file_type() {
            Some(ft) if ft.is_file() => {}
            _ => continue,
        };

        let metadata = entry
            .metadata()
            .map_err(|e| DuplffError::ScanError(e.to_string()))?;
        let size = metadata.len();

        // Apply size filter
        if size < config.min_size {
            continue;
        }
        if let Some(max) = config.max_size {
            if size > max {
                continue;
            }
        }

        // Apply extension filter
        if let Some(ref exts) = config.extensions {
            let file_ext = entry.path().extension().and_then(|e| e.to_str());
            match file_ext {
                Some(ext) if exts.iter().any(|e| e.eq_ignore_ascii_case(ext)) => {}
                _ => continue,
            }
        }

        let modified = metadata.modified().unwrap_or(std::time::UNIX_EPOCH);
        files.push(FileEntry {
            path: entry.into_path(),
            size,
            modified,
        });

        let count = counter.fetch_add(1, Ordering::Relaxed) + 1;
        if count.is_multiple_of(1000) {
            progress.on_scan_progress(count);
        }
    }

    let total = files.len();
    progress.on_scan_progress(total);
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::progress::NoopProgress;
    use std::fs;
    use tempfile::TempDir;

    fn make_test_tree() -> TempDir {
        let dir = TempDir::new().unwrap();
        // Create files of varying sizes
        fs::write(dir.path().join("a.txt"), "hello").unwrap(); // 5 bytes
        fs::write(dir.path().join("b.py"), "world!").unwrap(); // 6 bytes
        fs::create_dir(dir.path().join("sub")).unwrap();
        fs::write(dir.path().join("sub/c.rs"), "fn main() {}").unwrap(); // 13 bytes
        fs::write(dir.path().join("sub/d.txt"), "hi").unwrap(); // 2 bytes
                                                                // Empty file — should be skipped with min_size=1
        fs::write(dir.path().join("empty.txt"), "").unwrap();
        dir
    }

    #[test]
    fn scans_all_files_with_no_filters() {
        let dir = make_test_tree();
        let config = ScanConfig {
            roots: vec![dir.path().to_path_buf()],
            min_size: 1,
            ..ScanConfig::default()
        };
        let files = scan(&config, &NoopProgress).unwrap();
        assert_eq!(files.len(), 4); // excludes empty.txt
    }

    #[test]
    fn filters_by_extension() {
        let dir = make_test_tree();
        let config = ScanConfig {
            roots: vec![dir.path().to_path_buf()],
            extensions: Some(vec!["txt".into()]),
            min_size: 1,
            ..ScanConfig::default()
        };
        let files = scan(&config, &NoopProgress).unwrap();
        // a.txt (5b) and sub/d.txt (2b) — both >=1 byte with .txt extension
        assert_eq!(files.len(), 2);
        assert!(files.iter().all(|f| f.path.extension().unwrap() == "txt"));
    }

    #[test]
    fn filters_by_min_size() {
        let dir = make_test_tree();
        let config = ScanConfig {
            roots: vec![dir.path().to_path_buf()],
            min_size: 5,
            ..ScanConfig::default()
        };
        let files = scan(&config, &NoopProgress).unwrap();
        assert!(files.iter().all(|f| f.size >= 5));
    }

    #[test]
    fn excludes_matching_patterns() {
        let dir = make_test_tree();
        let config = ScanConfig {
            roots: vec![dir.path().to_path_buf()],
            min_size: 1,
            exclude_patterns: vec!["sub".to_string()],
            ..ScanConfig::default()
        };
        let files = scan(&config, &NoopProgress).unwrap();
        // sub/c.rs and sub/d.txt should be excluded
        assert!(files.iter().all(|f| !f.path.to_str().unwrap().contains("sub")));
        assert_eq!(files.len(), 2); // a.txt and b.py only
    }

    #[test]
    fn returns_correct_metadata() {
        let dir = make_test_tree();
        let config = ScanConfig {
            roots: vec![dir.path().to_path_buf()],
            min_size: 1,
            ..ScanConfig::default()
        };
        let files = scan(&config, &NoopProgress).unwrap();
        let a = files
            .iter()
            .find(|f| f.path.file_name().unwrap() == "a.txt")
            .unwrap();
        assert_eq!(a.size, 5);
    }
}
