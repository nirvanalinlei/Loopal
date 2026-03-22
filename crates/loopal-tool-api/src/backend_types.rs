//! Value types returned by [`Backend`](super::Backend) methods.
//!
//! These are deliberately simple structs so that tool crates only depend
//! on `loopal-tool-api` (a leaf crate) for their I/O interface.

/// Result of a file read operation.
#[derive(Debug, Clone)]
pub struct ReadResult {
    /// File content (with line-numbered formatting if applicable).
    pub content: String,
    /// Total number of lines in the original file.
    pub total_lines: usize,
}

/// Result of a file write operation.
#[derive(Debug, Clone)]
pub struct WriteResult {
    pub bytes_written: usize,
}

/// Result of a search-and-replace edit operation.
#[derive(Debug, Clone)]
pub struct EditResult {
    /// Number of replacements made.
    pub replacements: usize,
}

/// Result of a shell command execution.
#[derive(Debug, Clone)]
pub struct ExecResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

/// Result of an HTTP fetch operation.
#[derive(Debug, Clone)]
pub struct FetchResult {
    pub body: String,
    pub content_type: Option<String>,
    pub status: u16,
}

/// Metadata about a single file or directory.
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub size: u64,
    pub is_dir: bool,
    pub is_binary: bool,
    pub modified: Option<u64>,
}

/// Result of a directory listing.
#[derive(Debug, Clone)]
pub struct LsResult {
    pub entries: Vec<LsEntry>,
}

/// Single entry in a directory listing.
#[derive(Debug, Clone)]
pub struct LsEntry {
    pub name: String,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub size: u64,
    pub modified: Option<u64>,
    pub permissions: Option<u32>,
}

/// Result of a glob search.
#[derive(Debug, Clone)]
pub struct GlobResult {
    pub paths: Vec<String>,
    /// `true` when results were capped at the configured limit.
    pub truncated: bool,
}

/// Result of a grep search.
#[derive(Debug, Clone)]
pub struct GrepResult {
    pub matches: Vec<GrepMatch>,
    /// `true` when results were capped at the configured limit.
    pub truncated: bool,
}

/// Single match in a grep result.
#[derive(Debug, Clone)]
pub struct GrepMatch {
    pub path: String,
    pub line_number: usize,
    pub content: String,
}
