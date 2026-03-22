use thiserror::Error;

/// Errors returned by `Backend` trait methods.
///
/// Designed as a self-contained error type so that `Backend` consumers
/// (tool crates) can handle I/O failures without depending on platform details.
#[derive(Debug, Error)]
pub enum ToolIoError {
    #[error("path denied: {0}")]
    PathDenied(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("file too large: {path} ({size} bytes, limit {limit})")]
    TooLarge { path: String, size: u64, limit: u64 },

    #[error("binary file: {0}")]
    BinaryFile(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("exec failed: {0}")]
    ExecFailed(String),

    #[error("timeout after {0}ms")]
    Timeout(u64),

    #[error("network error: {0}")]
    Network(String),

    #[error("{0}")]
    Other(String),
}
