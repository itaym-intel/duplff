use thiserror::Error;

/// Unified error type for all duplff-core operations.
#[derive(Debug, Error)]
pub enum DuplffError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("scan error: {0}")]
    ScanError(String),

    #[error("hash error: {0}")]
    HashError(String),

    #[error("trash error: {0}")]
    TrashError(String),
}

/// Convenience alias used throughout duplff-core.
pub type Result<T> = std::result::Result<T, DuplffError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn io_error_converts() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "gone");
        let err: DuplffError = io_err.into();
        assert!(matches!(err, DuplffError::Io(_)));
        assert!(err.to_string().contains("gone"));
    }

    #[test]
    fn error_display_is_meaningful() {
        let err = DuplffError::ScanError("bad path".into());
        assert_eq!(err.to_string(), "scan error: bad path");
    }
}
