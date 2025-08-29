// lib.rs
#![forbid(unsafe_code)]

#[cfg(feature = "async")]
pub mod async_glob;
pub mod batch_io;
pub mod error;
pub mod options;
pub mod patterns;
pub mod predicates;
pub mod sync;
pub mod windows;

pub use crate::error::GlobError;
pub use crate::options::{GlobOptions, GlobOptionsBuilder};
pub use crate::patterns::Patterns;
pub use crate::predicates::Predicates;

use std::path::PathBuf;

/// Main facade for the PatternHunt library
///
/// This struct provides high-level APIs for both synchronous
/// and asynchronous file globbing with pattern matching.
pub struct PatternHunt;

impl PatternHunt {
    /// Performs synchronous glob pattern matching
    ///
    /// This method searches for files matching the specified patterns
    /// in the given root directories, with configurable options.
    ///
    /// # Arguments
    ///
    /// * `patterns` - Array of pattern strings to match
    /// * `roots` - Array of root directories to search in
    /// * `opts` - Configuration options for globbing
    ///
    /// # Returns
    ///
    /// `Ok(Vec<PathBuf>)` with matching paths, or `Err(GlobError)` on failure
    ///
    /// # Examples
    ///
    /// ```
    /// use patternhunt::{PatternHunt, GlobOptions};
    ///
    /// let options = GlobOptions::default();
    /// let results = PatternHunt::sync(
    ///     &["*.txt", "*.md"],
    ///     &["."],
    ///     options
    /// ).unwrap();
    /// ```
    pub fn sync(
        patterns: &[&str],
        roots: &[&str],
        opts: GlobOptions,
    ) -> Result<Vec<PathBuf>, GlobError> {
        let pats = Patterns::compile_many(patterns, &opts)?;
        let preds = opts.predicates.clone();
        let mut results = Vec::new();

        // Process each root directory
        for r in roots {
            let _root = std::path::Path::new(r);
            let mut v = crate::sync::glob_sync(pats.clone(), opts.clone(), preds.clone())?;
            results.append(&mut v);
        }

        Ok(results)
    }

    /// Creates a stream of results for asynchronous glob pattern matching
    ///
    /// This method returns a stream that asynchronously yields matching
    /// paths, suitable for use with async runtimes like Tokio.
    ///
    /// # Arguments
    ///
    /// * `patterns` - Array of pattern strings to match
    /// * `roots` - Array of root directories to search in
    /// * `opts` - Configuration options for globbing
    ///
    /// # Returns
    ///
    /// `Ok(impl Stream<Item = Result<PathBuf, GlobError>>)` on success,
    /// or `Err(GlobError)` if pattern compilation fails
    ///
    /// # Note
    ///
    /// Currently supports single-root operations. For multiple roots,
    /// consumers should call this method for each root directory.
    #[cfg(feature = "async")]
    pub fn stream(
        patterns: &[&str],
        _roots: &[&str],
        opts: GlobOptions,
    ) -> Result<impl futures::Stream<Item = Result<PathBuf, GlobError>>, GlobError> {
        let pats = Patterns::compile_many(patterns, &opts)?;
        let preds = opts.predicates.clone();

        // Simple single-root support for facade
        // Consumer can call for each root if needed
        Ok(crate::async_glob::glob_stream(pats, opts, preds))
    }
}
