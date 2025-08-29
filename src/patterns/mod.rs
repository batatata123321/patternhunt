// patterns/mod.rs
pub mod brace;
pub mod cache;
pub mod micromatch;

use crate::error::GlobError;
use crate::options::GlobOptions;
use globset::GlobSet;

/// Compiled patterns for efficient matching against paths
///
/// This struct combines both glob patterns and regex patterns
/// for flexible and efficient path matching.
#[derive(Clone)]
pub struct Patterns {
    pub set: GlobSet,
    pub regexes: Vec<regex::Regex>,
}

impl Patterns {
    /// Compiles multiple patterns into a Patterns instance
    ///
    /// This method handles brace expansion, regex patterns, and
    /// converts complex patterns to regex when necessary.
    ///
    /// # Arguments
    ///
    /// * `patterns` - Iterator of pattern strings
    /// * `opts` - Glob options for configuration
    ///
    /// # Returns
    ///
    /// `Ok(Patterns)` if successful, `Err(GlobError)` otherwise
    ///
    /// # Errors
    ///
    /// Returns `GlobError::PathTraversal` for patterns attempting path traversal
    /// Returns other `GlobError` variants for invalid patterns
    pub fn compile_many<I, S>(patterns: I, opts: &GlobOptions) -> Result<Self, GlobError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut builder = globset::GlobSetBuilder::new();
        let mut regexes = Vec::new();

        for pattern in patterns {
            let pattern_str = pattern.as_ref().trim();
            if pattern_str.is_empty() {
                continue;
            }

            // Protect against path traversal in patterns
            if pattern_str.contains("**/..") || pattern_str.contains("/../") {
                return Err(GlobError::PathTraversal);
            }

            // Process each pattern individually
            Self::process_pattern(pattern_str, &mut builder, &mut regexes, opts)?;
        }

        let set = builder
            .build()
            .map_err(|e| GlobError::InvalidPattern(e.to_string()))?;

        Ok(Self { set, regexes })
    }

    /// Processes a single pattern, handling brace expansion and type detection
    fn process_pattern(
        pattern: &str,
        builder: &mut globset::GlobSetBuilder,
        regexes: &mut Vec<regex::Regex>,
        _opts: &GlobOptions,
    ) -> Result<(), GlobError> {
        // Check if brace expansion is needed
        let expanded_patterns = if pattern.contains('{') && pattern.contains('}') {
            brace::expand(pattern)?
        } else {
            vec![pattern.to_string()]
        };

        for expanded in expanded_patterns {
            // Handle explicit regex patterns (prefixed with "re:")
            if let Some(regex_pattern) = expanded.strip_prefix("re:") {
                let re = cache::get_or_compile_regex(regex_pattern)?;
                regexes.push(re);
                continue;
            }

            // Determine if pattern requires regex conversion
            if Self::is_complex_pattern(&expanded) {
                // Convert complex patterns to regex
                let regex_pattern = micromatch::micromatch_to_regex(&expanded)?;
                let re = cache::get_or_compile_regex(&regex_pattern)?;
                regexes.push(re);
            } else {
                // Process as regular glob pattern
                Self::add_glob_pattern(&expanded, builder)?;
            }
        }

        Ok(())
    }

    /// Checks if a pattern contains advanced glob features requiring regex
    fn is_complex_pattern(pattern: &str) -> bool {
        // Check for extended glob features that require regex conversion
        pattern.contains('@')
            || pattern.contains('!')
            || pattern.contains('+')
            || pattern.contains('?')
            || pattern.contains('(')
            || pattern.contains(')')
            || pattern.contains('[')
            || pattern.contains(']')
            || pattern.contains('{')
            || pattern.contains('}')
            || pattern.contains('|')
    }

    /// Adds a glob pattern to the globset builder
    fn add_glob_pattern(
        pattern: &str,
        builder: &mut globset::GlobSetBuilder,
    ) -> Result<(), GlobError> {
        let glob =
            globset::Glob::new(pattern).map_err(|e| GlobError::InvalidPattern(e.to_string()))?;

        builder.add(glob);
        Ok(())
    }

    /// Checks if a path matches any of the compiled patterns
    ///
    /// # Arguments
    ///
    /// * `path` - UTF-8 path to check
    ///
    /// # Returns
    ///
    /// `true` if the path matches any pattern, `false` otherwise
    pub fn is_match(&self, path: &camino::Utf8PathBuf) -> bool {
        let path_str = path.as_str();

        // First check globset (usually faster)
        if !self.set.is_empty() && self.set.is_match(path_str) {
            return true;
        }

        // Then check regexes
        for re in &self.regexes {
            if re.is_match(path_str) {
                return true;
            }
        }

        false
    }

    /// Quickly checks if a path could potentially match any pattern
    ///
    /// This is a preliminary check before exact matching that can
    /// help avoid unnecessary work for obviously non-matching paths.
    ///
    /// # Arguments
    ///
    /// * `path` - UTF-8 path to check
    ///
    /// # Returns
    ///
    /// `true` if the path might match, `false` if it definitely won't
    pub fn could_match(&self, path: &camino::Utf8PathBuf) -> bool {
        self.set.is_match(path.as_str()) || !self.regexes.is_empty()
    }
}

/// Returns cache metrics for both glob and regex caches
pub fn cache_metrics() -> (cache::CacheMetrics, cache::CacheMetrics) {
    (cache::glob_cache_metrics(), cache::regex_cache_metrics())
}
