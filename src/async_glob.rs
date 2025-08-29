// async_glob.rs
#[cfg(feature = "async")]
use crate::{
    batch_io::BatchIO, error::GlobError, patterns::Patterns, predicates::Predicates, GlobOptions,
};
#[cfg(feature = "async")]
use async_stream::stream;
#[cfg(feature = "async")]
use camino::Utf8PathBuf;
#[cfg(feature = "async")]
use futures::Stream;
#[cfg(feature = "async")]
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
#[cfg(feature = "async")]
use tokio::{fs, sync::Semaphore, task};

#[cfg(feature = "async")]
/// Checks for symlink cycles during directory traversal
///
/// This function maintains a set of visited paths and detects cycles
/// when following symlinks to prevent infinite loops.
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

#[cfg(feature = "async")]
/// Checks if a path is allowed based on root directory restrictions
///
/// This function ensures that paths outside the specified root directory
/// are excluded from results for security reasons.
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

#[cfg(feature = "async")]
/// Creates a stream of glob pattern matching results
///
/// This function performs asynchronous directory traversal and pattern
/// matching, returning a stream that yields results as they are found.
///
/// # Arguments
///
/// * `patterns` - Compiled patterns to match against
/// * `opts` - Configuration options for globbing
/// * `predicates` - Optional predicates for filtering files
///
/// # Returns
///
/// A stream that yields `Result<PathBuf, GlobError>` values
pub fn glob_stream(
    patterns: Patterns,
    opts: GlobOptions,
    predicates: Option<Predicates>,
) -> impl Stream<Item = Result<PathBuf, GlobError>> {
    let semaphore = Arc::new(Semaphore::new(opts.max_inflight));
    let patterns = Arc::new(patterns);
    let predicates = Arc::new(predicates);
    let batch_io = Arc::new(BatchIO::new(1000, opts.follow_symlinks));
    let root = opts.root_dir.clone().unwrap_or_else(|| PathBuf::from("."));

    stream! {
        let mut visited_links = HashSet::new();
        let mut stack = vec![(root, 0)]; // (directory, depth)

        while let Some((dir, depth)) = stack.pop() {
            let mut rd = match fs::read_dir(&dir).await {
                Ok(rd) => rd,
                Err(e) => {
                    yield Err(GlobError::Io(e));
                    continue;
                }
            };

            loop {
                let entry = match rd.next_entry().await {
                    Ok(Some(entry)) => entry,
                    Ok(None) => break,
                    Err(e) => {
                        yield Err(GlobError::Io(e));
                        break;
                    }
                };

                let path = entry.path();

                if !is_path_allowed(&path, &opts.root_dir) {
                    continue;
                }

                if opts.follow_symlinks && check_for_cycles(&path, &mut visited_links) {
                    continue;
                }

                let file_type = match entry.file_type().await {
                    Ok(ft) => ft,
                    Err(e) => {
                        yield Err(GlobError::Io(e));
                        continue;
                    }
                };

                let is_dir = file_type.is_dir();
                let is_symlink = file_type.is_symlink();

                if is_symlink && !opts.follow_symlinks {
                    continue;
                }

                if is_dir {
                    if let Some(max_depth) = opts.max_depth {
                        if depth >= max_depth {
                            continue;
                        }
                    }
                    stack.push((path.clone(), depth + 1));
                    continue;
                }

                // For files, process asynchronously with bounded concurrency
                let patterns_clone = patterns.clone();
                let predicates_clone = predicates.clone();
                let batch_io_clone = batch_io.clone();
                let path_clone = path.clone();
                let semaphore_clone = semaphore.clone();

                // Acquire semaphore permit with timeout
                let permit = match tokio::time::timeout(
                    opts.timeout.unwrap_or(Duration::from_secs(30)),
                    semaphore_clone.acquire_owned()
                ).await {
                    Ok(Ok(permit)) => permit,
                    Ok(Err(_)) => continue, // Semaphore closed
                    Err(_) => continue,     // Timeout
                };

                // Spawn blocking task for CPU-intensive operations
                let join_handle = task::spawn_blocking(move || {
                    let _permit = permit; // Hold permit for task duration

                    let utf8_path = match Utf8PathBuf::from_path_buf(path_clone.clone()) {
                        Ok(p) => p,
                        Err(_) => return Ok(None), // Skip non-UTF8 paths
                    };

                    // Pattern matching
                    if !patterns_clone.is_match(&utf8_path) {
                        return Ok(None);
                    }

                    // Predicate filtering
                    if let Some(preds) = &*predicates_clone {
                        let meta = match batch_io_clone.stat(&path_clone) {
                            Ok(meta) => meta,
                            Err(e) => return Err(e),
                        };
                        if !preds.matches(&meta) {
                            return Ok(None);
                        }
                    }

                    Ok(Some(path_clone))
                });

                // Handle task results
                match join_handle.await {
                    Ok(Ok(Some(file))) => yield Ok(file),
                    Ok(Ok(None)) => {}, // No match
                    Ok(Err(e)) => yield Err(e),
                    Err(e) => yield Err(GlobError::Other(format!("Task failed: {}", e))),
                }
            }
        }
    }
}
