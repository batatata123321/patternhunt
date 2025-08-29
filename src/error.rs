// error.rs
use std::io;
use thiserror::Error;
use walkdir;

/// Error types for glob operations
///
/// This enum represents all possible errors that can occur during
/// glob pattern matching and filesystem operations.
#[derive(Error, Debug)]
pub enum GlobError {
    /// I/O error from filesystem operations
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// Regex compilation error
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    /// Invalid pattern syntax
    #[error("Invalid pattern: {0}")]
    InvalidPattern(String),

    /// Walkdir traversal error
    #[error("Walkdir error: {0}")]
    Walkdir(#[from] walkdir::Error),

    /// Other unspecified errors
    #[error("Other error: {0}")]
    Other(String),

    /// Brace expansion exceeded maximum depth
    #[error("Brace expansion exceeded maximum depth")]
    BraceExpansionDepth,

    /// Brace expansion exceeded maximum number of expansions
    #[error("Brace expansion exceeded maximum expansions")]
    BraceExpansionCount,

    /// Regex pattern too complex or too long
    #[error("Regex pattern too complex or long")]
    RegexTooComplex,

    /// Path traversal attempt detected and blocked
    #[error("Path traversal not allowed")]
    PathTraversal,

    /// Symlink cycle detected during traversal
    #[error("Symlink cycle detected")]
    SymlinkCycle,

    /// Operation timed out
    #[error("Operation timed out")]
    Timeout,

    /// Permission denied for file access
    #[error("Permission denied")]
    PermissionDenied,
}
