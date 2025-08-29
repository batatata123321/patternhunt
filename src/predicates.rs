// predicates.rs
use std::{fs::Metadata, time::SystemTime};

/// File type predicates for filtering
///
/// This enum allows filtering files based on their type
/// during glob pattern matching operations.
#[derive(Clone, Debug)]
pub enum FileType {
    /// Regular files
    File,
    /// Directories
    Dir,
    /// Symbolic links
    Symlink,
}

/// Predicates for filtering files based on metadata
///
/// This struct provides a flexible way to filter files based on
/// various attributes like size, type, and timestamps.
#[derive(Clone, Debug)]
pub struct Predicates {
    /// Minimum file size in bytes
    pub min_size: Option<u64>,

    /// Maximum file size in bytes
    pub max_size: Option<u64>,

    /// Required file type
    pub file_type: Option<FileType>,

    /// Last modified after this time
    pub mtime_after: Option<SystemTime>,

    /// Last modified before this time
    pub mtime_before: Option<SystemTime>,

    /// Created after this time
    pub ctime_after: Option<SystemTime>,

    /// Created before this time
    pub ctime_before: Option<SystemTime>,

    /// Whether to follow symlinks for metadata checks
    pub follow_symlinks: bool,
}

impl Predicates {
    /// Checks if file metadata matches all predicates
    ///
    /// This method evaluates all configured predicates against
    /// the file metadata and returns true only if all match.
    ///
    /// # Arguments
    ///
    /// * `meta` - File metadata to evaluate
    ///
    /// # Returns
    ///
    /// `true` if all predicates match, `false` otherwise
    pub fn matches(&self, meta: &Metadata) -> bool {
        // Size predicates
        if let Some(min) = self.min_size {
            if meta.len() < min {
                return false;
            }
        }

        if let Some(max) = self.max_size {
            if meta.len() > max {
                return false;
            }
        }

        // File type predicate
        if let Some(ft) = &self.file_type {
            match ft {
                FileType::File if !meta.is_file() => return false,
                FileType::Dir if !meta.is_dir() => return false,
                FileType::Symlink if !meta.file_type().is_symlink() => return false,
                _ => {}
            }
        }

        // Modification time predicates
        if let Ok(mtime) = meta.modified() {
            if let Some(after) = self.mtime_after {
                if mtime < after {
                    return false;
                }
            }
            if let Some(before) = self.mtime_before {
                if mtime > before {
                    return false;
                }
            }
        }

        // Creation time predicates
        if let Ok(ctime) = meta.created() {
            if let Some(after) = self.ctime_after {
                if ctime < after {
                    return false;
                }
            }
            if let Some(before) = self.ctime_before {
                if ctime > before {
                    return false;
                }
            }
        }

        true
    }
}
