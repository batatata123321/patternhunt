// options.rs
use crate::predicates::Predicates;
use std::{path::PathBuf, time::Duration};

/// Configuration options for glob operations
///
/// This struct allows fine-grained control over globbing behavior,
/// including symlink handling, depth limits, and filtering predicates.
#[derive(Clone, Debug)]
pub struct GlobOptions {
    /// Whether to follow symbolic links during traversal
    pub follow_symlinks: bool,

    /// Maximum directory depth to traverse (None for unlimited)
    pub max_depth: Option<usize>,

    /// Whether to use case-sensitive matching
    pub case_sensitive: bool,

    /// Maximum number of concurrent operations for async globbing
    pub max_inflight: usize,

    /// Timeout for individual operations
    pub timeout: Option<Duration>,

    /// Predicates for filtering files based on metadata
    pub predicates: Option<Predicates>,

    /// Root directory to start globbing from
    pub root_dir: Option<PathBuf>,
}

impl Default for GlobOptions {
    fn default() -> Self {
        Self {
            follow_symlinks: false,
            max_depth: None,
            case_sensitive: cfg!(not(windows)), // Case-insensitive by default on Windows
            max_inflight: 64,
            timeout: None,
            predicates: None,
            root_dir: None,
        }
    }
}

/// Builder for GlobOptions for fluent configuration
///
/// This builder pattern allows for clean, readable configuration
/// of glob options with method chaining.
pub struct GlobOptionsBuilder(GlobOptions);

impl Default for GlobOptionsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobOptionsBuilder {
    /// Creates a new builder with default options
    pub fn new() -> Self {
        Self(GlobOptions::default())
    }

    /// Sets whether to follow symbolic links
    pub fn follow_symlinks(mut self, v: bool) -> Self {
        self.0.follow_symlinks = v;
        self
    }

    /// Sets the maximum directory depth to traverse
    pub fn max_depth(mut self, d: usize) -> Self {
        self.0.max_depth = Some(d);
        self
    }

    /// Sets case-sensitive matching behavior
    pub fn case_sensitive(mut self, v: bool) -> Self {
        self.0.case_sensitive = v;
        self
    }

    /// Sets the maximum number of concurrent operations for async globbing
    pub fn max_inflight(mut self, v: usize) -> Self {
        self.0.max_inflight = v;
        self
    }

    /// Sets the timeout for individual operations
    pub fn timeout(mut self, t: Duration) -> Self {
        self.0.timeout = Some(t);
        self
    }

    /// Sets the predicates for file filtering
    pub fn predicates(mut self, p: Predicates) -> Self {
        self.0.predicates = Some(p);
        self
    }

    /// Sets the root directory for globbing
    pub fn root_dir(mut self, dir: PathBuf) -> Self {
        self.0.root_dir = Some(dir);
        self
    }

    /// Builds the final GlobOptions instance
    pub fn build(self) -> GlobOptions {
        self.0
    }
}
