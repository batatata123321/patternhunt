// sync.rs
use crate::{
    batch_io::BatchIO, error::GlobError, patterns::Patterns, predicates::Predicates, GlobOptions,
};
use camino::Utf8PathBuf;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

/// Checks for symlink cycles during directory traversal
///
/// This function maintains a set of visited paths and detects cycles
/// when following symlinks, which is crucial to prevent infinite loops.
///
/// # Arguments
///
/// * `path` - The current path being visited
/// * `visited` - Mutable reference to the set of visited paths
///
/// # Returns
///
/// `true` if a cycle is detected, `false` otherwise
fn check_for_cycles(path: &Path, visited: &mut HashSet<PathBuf>) -> bool {
    if visited.contains(path) {
        return true;
    }
    visited.insert(path.to_path_buf());
    false
}

/// Checks if a path is allowed based on root directory restrictions
///
/// This function ensures that paths outside the specified root directory
/// are excluded from results, providing basic security against path traversal.
///
/// # Arguments
///
/// * `path` - The path to check
/// * `root_dir` - Optional root directory restriction
///
/// # Returns
///
/// `true` if the path is allowed, `false` otherwise
fn is_path_allowed(path: &Path, root_dir: &Option<PathBuf>) -> bool {
    if let Some(root) = root_dir {
        path.starts_with(root)
    } else {
        true
    }
}

/// Performs synchronous glob pattern matching
///
/// This function traverses the directory tree synchronously using WalkDir,
/// matches paths against compiled patterns, and applies predicate filters.
///
/// # Arguments
///
/// * `patterns` - Compiled patterns to match against
/// * `opts` - Configuration options for globbing
/// * `predicates` - Optional predicates for filtering files
///
/// # Returns
///
/// `Ok(Vec<PathBuf>)` with matching paths, or `Err(GlobError)` on failure
///
/// # Errors
///
/// Returns `GlobError` for I/O errors, permission denied, symlink cycles,
/// and other issues during filesystem traversal.
pub fn glob_sync(
    patterns: Patterns,
    opts: GlobOptions,
    predicates: Option<Predicates>,
) -> Result<Vec<PathBuf>, GlobError> {
    let mut results = Vec::new();
    let root = opts.root_dir.clone().unwrap_or_else(|| PathBuf::from("."));
    let mut visited_links = HashSet::new();
    let batch_io = BatchIO::new(1000, opts.follow_symlinks);

    // Use WalkDir for efficient directory traversal
    for entry in WalkDir::new(&root)
        .follow_links(opts.follow_symlinks)
        .same_file_system(true)
        .max_depth(opts.max_depth.unwrap_or(usize::MAX))
    {
        let dent = entry.map_err(GlobError::Walkdir)?;
        let p = dent.path();

        // Check path restrictions
        if !is_path_allowed(p, &opts.root_dir) {
            continue;
        }

        // Check for symlink cycles if following symlinks
        if opts.follow_symlinks && check_for_cycles(p, &mut visited_links) {
            return Err(GlobError::SymlinkCycle);
        }

        // Skip directories (we're only interested in files)
        if p.is_dir() {
            continue;
        }

        // Convert to UTF-8 path for pattern matching
        if let Ok(up) = Utf8PathBuf::from_path_buf(p.to_path_buf()) {
            // Pattern matching
            if !patterns.is_match(&up) {
                continue;
            }

            // Predicate filtering
            if let Some(pred) = &predicates {
                let meta = batch_io.stat(p)?;
                if !pred.matches(&meta) {
                    continue;
                }
            }

            results.push(p.to_path_buf());
        }
    }

    Ok(results)
}
