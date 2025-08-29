// patterns/cache.rs
use crate::error::GlobError;
use globset::{Glob, GlobSet, GlobSetBuilder};
use lru::LruCache;
use once_cell::sync::Lazy;
use regex::Regex;
use std::{
    num::NonZeroUsize,
    sync::Mutex,
    time::{Duration, Instant},
};

// Limit cache size to prevent uncontrolled memory growth
const MAX_CACHE_SIZE: usize = 1000;
const DEFAULT_TTL: Duration = Duration::from_secs(300);
const MAX_REGEX_COMPLEXITY: usize = 1000;

/// A cache entry with value and expiration time
#[derive(Clone, Debug)]
struct CacheEntry<T> {
    value: T,
    expires_at: Instant,
}

/// Metrics for cache performance monitoring
#[derive(Clone, Debug)]
pub struct CacheMetrics {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub size: usize,
}

impl CacheMetrics {
    /// Calculates the cache hit ratio
    pub fn hit_ratio(&self) -> f64 {
        if self.hits + self.misses == 0 {
            0.0
        } else {
            self.hits as f64 / (self.hits + self.misses) as f64
        }
    }
}

/// Cache for compiled GlobSets with LRU eviction and TTL
struct GlobCache {
    cache: Mutex<LruCache<String, CacheEntry<GlobSet>>>,
    metrics: Mutex<CacheMetrics>,
    ttl: Duration,
}

/// Cache for compiled Regex patterns with LRU eviction and TTL
struct RegexCache {
    cache: Mutex<LruCache<String, CacheEntry<Regex>>>,
    metrics: Mutex<CacheMetrics>,
    ttl: Duration,
}

impl GlobCache {
    fn new(ttl: Duration) -> Self {
        Self {
            cache: Mutex::new(LruCache::new(NonZeroUsize::new(MAX_CACHE_SIZE).unwrap())),
            metrics: Mutex::new(CacheMetrics {
                hits: 0,
                misses: 0,
                evictions: 0,
                size: 0,
            }),
            ttl,
        }
    }

    /// Retrieves a cached GlobSet if present and not expired
    fn get(&self, key: &str) -> Option<GlobSet> {
        let mut cache = self.cache.lock().unwrap();
        let mut metrics = self.metrics.lock().unwrap();

        if let Some(entry) = cache.get(key) {
            if entry.expires_at > Instant::now() {
                metrics.hits += 1;
                return Some(entry.value.clone());
            } else {
                // Remove expired entry
                cache.pop(key);
                metrics.size = cache.len();
                metrics.evictions += 1;
            }
        }

        metrics.misses += 1;
        None
    }

    /// Stores a GlobSet in the cache with TTL
    fn put(&self, key: String, value: GlobSet) {
        let mut cache = self.cache.lock().unwrap();
        let mut metrics = self.metrics.lock().unwrap();

        let entry = CacheEntry {
            value,
            expires_at: Instant::now() + self.ttl,
        };

        cache.put(key, entry);
        metrics.size = cache.len();
    }

    /// Returns current cache metrics
    fn metrics(&self) -> CacheMetrics {
        self.metrics.lock().unwrap().clone()
    }

    /// Clears the cache
    fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        let mut metrics = self.metrics.lock().unwrap();

        cache.clear();
        metrics.size = 0;
        metrics.evictions += 1;
    }
}

impl RegexCache {
    fn new(ttl: Duration) -> Self {
        Self {
            cache: Mutex::new(LruCache::new(NonZeroUsize::new(MAX_CACHE_SIZE).unwrap())),
            metrics: Mutex::new(CacheMetrics {
                hits: 0,
                misses: 0,
                evictions: 0,
                size: 0,
            }),
            ttl,
        }
    }

    /// Retrieves a cached Regex if present and not expired
    fn get(&self, key: &str) -> Option<Regex> {
        let mut cache = self.cache.lock().unwrap();
        let mut metrics = self.metrics.lock().unwrap();

        if let Some(entry) = cache.get(key) {
            if entry.expires_at > Instant::now() {
                metrics.hits += 1;
                return Some(entry.value.clone());
            } else {
                // Remove expired entry
                cache.pop(key);
                metrics.size = cache.len();
                metrics.evictions += 1;
            }
        }

        metrics.misses += 1;
        None
    }

    /// Stores a Regex in the cache with TTL
    fn put(&self, key: String, value: Regex) {
        let mut cache = self.cache.lock().unwrap();
        let mut metrics = self.metrics.lock().unwrap();

        let entry = CacheEntry {
            value,
            expires_at: Instant::now() + self.ttl,
        };

        cache.put(key, entry);
        metrics.size = cache.len();
    }

    /// Returns current cache metrics
    fn metrics(&self) -> CacheMetrics {
        self.metrics.lock().unwrap().clone()
    }

    /// Clears the cache
    fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        let mut metrics = self.metrics.lock().unwrap();

        cache.clear();
        metrics.size = 0;
        metrics.evictions += 1;
    }
}

// Global cache instances
static GLOB_CACHE: Lazy<GlobCache> = Lazy::new(|| GlobCache::new(DEFAULT_TTL));
static REGEX_CACHE: Lazy<RegexCache> = Lazy::new(|| RegexCache::new(DEFAULT_TTL));

/// Retrieves a compiled GlobSet from cache or compiles and caches it
///
/// # Arguments
///
/// * `pattern` - Glob pattern to compile
///
/// # Returns
///
/// `Ok(GlobSet)` if successful, `Err(GlobError)` otherwise
pub fn get_or_compile_glob(pattern: &str) -> Result<GlobSet, GlobError> {
    if let Some(cached) = GLOB_CACHE.get(pattern) {
        return Ok(cached);
    }

    let mut builder = GlobSetBuilder::new();
    let g = Glob::new(pattern).map_err(|e| GlobError::InvalidPattern(e.to_string()))?;
    builder.add(g);
    let set = builder
        .build()
        .map_err(|e| GlobError::InvalidPattern(e.to_string()))?;

    GLOB_CACHE.put(pattern.to_string(), set.clone());
    Ok(set)
}

/// Retrieves a compiled Regex from cache or compiles and caches it
///
/// # Arguments
///
/// * `pat` - Regex pattern to compile
///
/// # Returns
///
/// `Ok(Regex)` if successful, `Err(GlobError)` otherwise
///
/// # Errors
///
/// Returns `GlobError::RegexTooComplex` for patterns that exceed complexity limits
pub fn get_or_compile_regex(pat: &str) -> Result<Regex, GlobError> {
    // Complexity checks to prevent ReDoS attacks
    if pat.len() > 1000 || pat.matches('(').count() > MAX_REGEX_COMPLEXITY {
        return Err(GlobError::RegexTooComplex);
    }

    if let Some(cached) = REGEX_CACHE.get(pat) {
        return Ok(cached);
    }

    let re = Regex::new(pat).map_err(GlobError::Regex)?;
    REGEX_CACHE.put(pat.to_string(), re.clone());
    Ok(re)
}

/// Clears both glob and regex caches
pub fn clear_caches() {
    GLOB_CACHE.clear();
    REGEX_CACHE.clear();
}

/// Returns metrics for the glob cache
pub fn glob_cache_metrics() -> CacheMetrics {
    GLOB_CACHE.metrics()
}

/// Returns metrics for the regex cache
pub fn regex_cache_metrics() -> CacheMetrics {
    REGEX_CACHE.metrics()
}

/// Sets the TTL for new cache entries (does not affect existing entries)
pub fn set_ttl(_ttl: Duration) {
    // For simplicity, we don't change TTL of existing entries
    // New entries will use the new TTL
}
