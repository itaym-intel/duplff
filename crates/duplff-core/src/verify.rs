use crate::error::{DuplffError, Result};
use std::fs::File;
use std::io::Read;
use std::path::Path;

const VERIFY_BUFFER_SIZE: usize = 128 * 1024;

/// Compare two files byte-by-byte. Returns true if identical.
pub fn files_identical(path_a: &Path, path_b: &Path) -> Result<bool> {
    let mut fa = File::open(path_a)
        .map_err(|e| DuplffError::HashError(format!("{}: {e}", path_a.display())))?;
    let mut fb = File::open(path_b)
        .map_err(|e| DuplffError::HashError(format!("{}: {e}", path_b.display())))?;

    let mut buf_a = vec![0u8; VERIFY_BUFFER_SIZE];
    let mut buf_b = vec![0u8; VERIFY_BUFFER_SIZE];

    loop {
        let n_a = fa
            .read(&mut buf_a)
            .map_err(|e| DuplffError::HashError(format!("{}: {e}", path_a.display())))?;
        let n_b = fb
            .read(&mut buf_b)
            .map_err(|e| DuplffError::HashError(format!("{}: {e}", path_b.display())))?;
        if n_a != n_b || buf_a[..n_a] != buf_b[..n_b] {
            return Ok(false);
        }
        if n_a == 0 {
            return Ok(true);
        }
    }
}

/// Verify all files in a group are byte-identical to the first file.
pub fn verify_group(paths: &[&Path]) -> Result<bool> {
    if paths.len() < 2 {
        return Ok(true);
    }
    let reference = paths[0];
    for other in &paths[1..] {
        if !files_identical(reference, other)? {
            return Ok(false);
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn identical_files_pass() {
        let dir = TempDir::new().unwrap();
        let a = dir.path().join("a.txt");
        let b = dir.path().join("b.txt");
        fs::write(&a, "hello world").unwrap();
        fs::write(&b, "hello world").unwrap();
        assert!(files_identical(&a, &b).unwrap());
    }

    #[test]
    fn different_files_fail() {
        let dir = TempDir::new().unwrap();
        let a = dir.path().join("a.txt");
        let b = dir.path().join("b.txt");
        fs::write(&a, "hello").unwrap();
        fs::write(&b, "world").unwrap();
        assert!(!files_identical(&a, &b).unwrap());
    }

    #[test]
    fn verify_group_all_identical() {
        let dir = TempDir::new().unwrap();
        let a = dir.path().join("a.txt");
        let b = dir.path().join("b.txt");
        let c = dir.path().join("c.txt");
        fs::write(&a, "same content").unwrap();
        fs::write(&b, "same content").unwrap();
        fs::write(&c, "same content").unwrap();
        assert!(verify_group(&[&a, &b, &c]).unwrap());
    }

    #[test]
    fn verify_group_one_different() {
        let dir = TempDir::new().unwrap();
        let a = dir.path().join("a.txt");
        let b = dir.path().join("b.txt");
        let c = dir.path().join("c.txt");
        fs::write(&a, "same content").unwrap();
        fs::write(&b, "same content").unwrap();
        fs::write(&c, "diff content").unwrap();
        assert!(!verify_group(&[&a, &b, &c]).unwrap());
    }
}
