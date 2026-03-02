// File hashing for duplff-core

use crate::error::{DuplffError, Result};
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Size of the partial hash sample: first 4KB.
const PARTIAL_HASH_SIZE: usize = 4096;

/// Buffer size for streaming full-file hashing: 128KB.
const HASH_BUFFER_SIZE: usize = 128 * 1024;

/// Compute a BLAKE3 hash of the first 4KB of a file.
///
/// For files smaller than 4KB, hashes the entire content.
/// This is used as a cheap pre-filter before full hashing.
pub fn partial_hash(path: &Path) -> Result<[u8; 32]> {
    let file = File::open(path)
        .map_err(|e| DuplffError::HashError(format!("{}: {}", path.display(), e)))?;
    let mut buf = Vec::with_capacity(PARTIAL_HASH_SIZE);
    file.take(PARTIAL_HASH_SIZE as u64)
        .read_to_end(&mut buf)
        .map_err(|e| DuplffError::HashError(format!("{}: {}", path.display(), e)))?;
    Ok(*blake3::hash(&buf).as_bytes())
}

/// Compute a full BLAKE3 hash of a file's entire content.
///
/// Uses 128KB buffered reads for throughput.
pub fn full_hash(path: &Path) -> Result<[u8; 32]> {
    let mut file = File::open(path)
        .map_err(|e| DuplffError::HashError(format!("{}: {}", path.display(), e)))?;
    let mut hasher = blake3::Hasher::new();
    let mut buf = vec![0u8; HASH_BUFFER_SIZE];
    loop {
        let n = file
            .read(&mut buf)
            .map_err(|e| DuplffError::HashError(format!("{}: {}", path.display(), e)))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(*hasher.finalize().as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn partial_hash_of_small_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("small.txt");
        fs::write(&path, "hello world").unwrap();
        let hash = partial_hash(&path).unwrap();
        // Same content must produce same hash
        let hash2 = partial_hash(&path).unwrap();
        assert_eq!(hash, hash2);
    }

    #[test]
    fn partial_hash_differs_for_different_content() {
        let dir = TempDir::new().unwrap();
        let a = dir.path().join("a.txt");
        let b = dir.path().join("b.txt");
        fs::write(&a, "hello").unwrap();
        fs::write(&b, "world").unwrap();
        assert_ne!(partial_hash(&a).unwrap(), partial_hash(&b).unwrap());
    }

    #[test]
    fn full_hash_matches_blake3_reference() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.txt");
        let content = b"deterministic content for hashing";
        fs::write(&path, content).unwrap();
        let hash = full_hash(&path).unwrap();
        let expected = blake3::hash(content);
        assert_eq!(hash, *expected.as_bytes());
    }

    #[test]
    fn full_hash_of_large_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("large.bin");
        // 1MB of repeated bytes
        let content = vec![0xABu8; 1024 * 1024];
        fs::write(&path, &content).unwrap();
        let hash = full_hash(&path).unwrap();
        let expected = blake3::hash(&content);
        assert_eq!(hash, *expected.as_bytes());
    }

    #[test]
    fn partial_hash_same_for_files_sharing_first_4kb() {
        let dir = TempDir::new().unwrap();
        let a = dir.path().join("a.bin");
        let b = dir.path().join("b.bin");
        // Same first 4KB, different after
        let content_a = vec![0u8; 8192];
        let mut content_b = vec![0u8; 8192];
        content_b[4096] = 0xFF; // differ after first 4KB
        fs::write(&a, &content_a).unwrap();
        fs::write(&b, &content_b).unwrap();
        // Partial hashes should be equal (only first 4KB)
        assert_eq!(partial_hash(&a).unwrap(), partial_hash(&b).unwrap());
        // Full hashes should differ
        assert_ne!(full_hash(&a).unwrap(), full_hash(&b).unwrap());
    }
}
