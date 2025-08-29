// windows.rs
/// Ensures a path has the long path prefix on Windows
///
/// On Windows, paths longer than MAX_PATH need the "\\?\" prefix
/// to avoid path length limitations. This function adds the prefix
/// if it's not already present.
///
/// # Arguments
///
/// * `p` - The path to process
///
/// # Returns
///
/// The path with the long path prefix if needed
#[cfg(windows)]
pub fn ensure_long_path_prefix(p: &std::path::Path) -> std::path::PathBuf {
    use std::path::PathBuf;
    let s = p.to_string_lossy();

    // Return unchanged if already has prefix
    if s.starts_with("\\\\?\\") {
        return p.to_path_buf();
    }

    // Add the long path prefix
    let mut pref = String::from("\\\\?\\");
    pref.push_str(&s);
    PathBuf::from(pref)
}

/// No-op implementation for non-Windows platforms
///
/// # Arguments
///
/// * `p` - The path to process
///
/// # Returns
///
/// The unchanged path
#[cfg(not(windows))]
pub fn ensure_long_path_prefix(p: &std::path::Path) -> std::path::PathBuf {
    p.to_path_buf()
}
