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
