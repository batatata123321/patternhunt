// batch_io.rs
use crate::error::GlobError;
use lru::LruCache;
use std::{
    fs,
    num::NonZeroUsize,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Duration, Instant},
};

/// Configuration for metadata caching
const METADATA_CACHE_TTL: Duration = Duration::from_secs(30);

/// A cached metadata entry with expiration timestamp
#[derive(Debug, Clone)]
struct CachedMetadata {
    metadata: fs::Metadata,
    expires_at: Instant,
}

/// Batch I/O operations with metadata caching
///
/// This struct provides efficient access to filesystem metadata
/// with LRU caching and configurable symlink following behavior.
#[derive(Debug)]
pub struct BatchIO {
    metadata_cache: Mutex<LruCache<PathBuf, CachedMetadata>>,
    follow_symlinks: bool,
}

impl BatchIO {
    /// Creates a new BatchIO instance with specified cache size and symlink behavior
    ///
    /// # Arguments
    ///
    /// * `cache_size` - Maximum number of metadata entries to cache
    /// * `follow_symlinks` - Whether to follow symlinks when retrieving metadata
    ///
    /// # Returns
    ///
    /// A new BatchIO instance
    pub fn new(cache_size: usize, follow_symlinks: bool) -> Self {
        Self {
            metadata_cache: Mutex::new(LruCache::new(NonZeroUsize::new(cache_size).unwrap())),
            follow_symlinks,
        }
    }

    /// Retrieves metadata for a path with caching
    ///
    /// This method checks the cache first, and if not found or expired,
    /// queries the filesystem. Also performs permission checks.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to retrieve metadata for
    ///
    /// # Returns
    ///
    /// `Ok(Metadata)` if successful, `Err(GlobError)` otherwise
    ///
    /// # Errors
    ///
    /// Returns `GlobError::PermissionDenied` if file is read-only
    /// Returns `GlobError::Io` for I/O errors
    /// Returns `GlobError::Other` for symlinks when not allowed
    pub fn stat(&self, path: &Path) -> Result<fs::Metadata, GlobError> {
        let mut cache = self.metadata_cache.lock().unwrap();

        // Check cache first
        if let Some(cached) = cache.get(path) {
            if cached.expires_at > Instant::now() {
                return Ok(cached.metadata.clone());
            }
            // Remove expired entry
            cache.pop(path);
        }

        // Check symlink restrictions
        if !self.follow_symlinks && path.is_symlink() {
            return Err(GlobError::Other("Symlinks not allowed".into()));
        }

        // Query filesystem
        let meta = fs::metadata(path).map_err(GlobError::Io)?;

        // Permission check
        if meta.permissions().readonly() {
            return Err(GlobError::PermissionDenied);
        }

        // Cache the result
        let cached_meta = CachedMetadata {
            metadata: meta.clone(),
            expires_at: Instant::now() + METADATA_CACHE_TTL,
        };
        cache.put(path.to_path_buf(), cached_meta);

        Ok(meta)
    }

    /// Retrieves metadata for a symlink without following it
    ///
    /// This method always queries the filesystem directly without caching,
    /// as symlink metadata is typically less frequently accessed.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the symlink
    ///
    /// # Returns
    ///
    /// `Ok(Metadata)` if successful, `Err(GlobError)` otherwise
    pub fn stat_symlink(&self, path: &Path) -> Result<fs::Metadata, GlobError> {
        fs::symlink_metadata(path).map_err(GlobError::Io)
    }

    /// Clears the metadata cache
    ///
    /// Useful when filesystem changes are expected and cached data
    /// might become stale.
    pub fn clear_cache(&self) {
        let mut cache = self.metadata_cache.lock().unwrap();
        cache.clear();
    }
}
