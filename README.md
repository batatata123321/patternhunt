# PatternHunt - Advanced File Globbing Library for Rust

[![Crates.io](https://img.shields.io/crates/v/patternhunt.svg)](https://crates.io/crates/patternhunt)
[![Documentation](https://docs.rs/patternhunt/badge.svg)](https://docs.rs/patternhunt)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)

A high-performance, feature-rich glob pattern matching library for Rust, designed for efficient and flexible file path matching with both synchronous and asynchronous APIs.

## Overview

This library provides robust glob pattern matching capabilities with support for advanced features such as brace expansion, extended glob patterns (extglobs), regex integration, and metadata-based filtering. It is optimized for performance with caching mechanisms for compiled patterns and filesystem metadata, making it suitable for both small-scale and large-scale filesystem operations.

The library is built with safety and security in mind, including protections against path traversal, symlink cycles, and excessive resource usage (e.g., preventing ReDoS attacks and stack overflows).

## Features

- **Brace Expansion**: Supports nested brace patterns (e.g., `file.{txt,md}`) and numeric ranges (e.g., `test{1..3}`).
- **Extended Glob Patterns**: Handles advanced glob patterns like `@(pattern)`, `*(pattern)`, `+(pattern)`, `?(pattern)`, and `!(pattern)` using the `micromatch` module.
- **Regex Integration**: Supports explicit regex patterns prefixed with `re:` and converts complex glob patterns to regex when needed.
- **Synchronous and Asynchronous APIs**: Provides both `sync` and `async` globbing functions for flexible integration into different application types.
- **Metadata Filtering**: Allows filtering of matched files based on size, file type, and timestamps using the `Predicates` struct.
- **Caching**: Implements LRU caching for compiled glob patterns, regexes, and filesystem metadata to improve performance.
- **Security Features**:
  - Path traversal protection.
  - Symlink cycle detection.
  - Configurable symlink following behavior.
  - Limits on brace expansion depth and count to prevent DoS attacks.
  - Regex complexity checks to prevent ReDoS attacks.
- **Configurable Options**: Fine-grained control over globbing behavior through `GlobOptions`, including case sensitivity, maximum depth, and concurrency limits.
- **Performance Monitoring**: Provides cache metrics (hits, misses, evictions) for performance tuning.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
patternhunt = "0.1"
```

Ensure you have the required dependencies installed, including `globset`, `regex`, `lru`, `once_cell`, `camino`, `walkdir`, and `tokio` (for async features).

## Usage

### Basic Example (Synchronous)

```rust
use patternhunt::{PatternHunt, GlobOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Search for Rust and Markdown files
    let results = PatternHunt::sync(
        &["*.rs", "*.md", "Cargo.toml"],
        &["."],
        GlobOptions::default()
    )?;

    for path in results {
        println!("Found: {}", path.display());
    }

    Ok(())
}
```

### Asynchronous Example

```rust
use patternhunt::{PatternHunt, GlobOptions};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a stream of results
    let mut stream = PatternHunt::stream(
        &["*.rs", "*.md"],
        &["."],
        GlobOptions::default()
    )?;

    // Process results as they're found
    while let Some(result) = stream.next().await {
        match result {
            Ok(path) => println!("Found: {}", path.display()),
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    Ok(())
}
```

### Using Brace Expansion

```rust
use patternhunt::{PatternHunt, GlobOptions};

fn brace_expansion() -> Result<(), Box<dyn std::error::Error>> {
    // Brace expansion with multiple extensions
    let results = PatternHunt::sync(
        &["src/**/*.{rs,toml,json}", "data/{2020..2023}-*.csv"],
        &["."],
        GlobOptions::default()
    )?;

    println!("Found {} files with brace expansion", results.len());
    Ok(())
}
```

### Regex Pattern Matchin

```rust
use patternhunt::{PatternHunt, GlobOptions};

fn regex_patterns() -> Result<(), Box<dyn std::error::Error>> {
    // Use regex patterns (prefixed with re:)
    let results = PatternHunt::sync(
        &["re:.*\\d{4}-\\d{2}-\\d{2}\\.log$", "re:^config\\.(yml|yaml|json)$"],
        &["."],
        GlobOptions::default()
    )?;

    println!("Found {} files with regex patterns", results.len());
    Ok(())
}
```

### Filtering with Predicates

```rust
use patternhunt::{PatternHunt, GlobOptions, GlobOptionsBuilder, Predicates, FileType};
use std::time::{SystemTime, Duration};

fn filtered_search() -> Result<(), Box<dyn std::error::Error>> {
    // Create predicates for filtering
    let predicates = Predicates {
        min_size: Some(1024),        // At least 1KB
        max_size: Some(1024 * 1024), // At most 1MB
        file_type: Some(FileType::File),
        mtime_after: Some(SystemTime::now() - Duration::from_secs(3600 * 24 * 7)), // Modified in last week
        mtime_before: None,
        ctime_after: None,
        ctime_before: None,
        follow_symlinks: false,
    };

    // Configure options with predicates
    let options = GlobOptionsBuilder::new()
        .predicates(predicates)
        .build();

    // Perform search with filtering
    let results = PatternHunt::sync(
        &["**/*"],
        &["."],
        options
    )?;

    println!("Found {} matching files", results.len());
    Ok(())
}
```

### Complex Pattern Matching

```rust
use patternhunt::{PatternHunt, GlobOptions};

fn complex_patterns() -> Result<(), Box<dyn std::error::Error>> {
    let options = GlobOptions::default();

    // Extended glob patterns
    let results = PatternHunt::sync(
        &["**/*.@(jpg|png|gif)", "**/!(temp)*.txt", "**/+([0-9]).log"],
        &["."],
        options
    )?;

    println!("Found {} files with complex patterns", results.len());
    Ok(())
}
```

### Multiple Root Directories

```rust
use patternhunt::{PatternHunt, GlobOptions};

fn multiple_roots() -> Result<(), Box<dyn std::error::Error>> {
    let options = GlobOptions::default();

    // Search in multiple directories
    let results = PatternHunt::sync(
        &["**/*.rs", "**/*.md"],
        &["src", "tests", "examples", "docs"],
        options
    )?;

    println!("Found {} files in multiple directories", results.len());
    Ok(())
}
```

### Depth-Limited Search

```rust
use patternhunt::{PatternHunt, GlobOptions, GlobOptionsBuilder};

fn depth_limited() -> Result<(), Box<dyn std::error::Error>> {
    // Limit search depth
    let options = GlobOptionsBuilder::new()
        .max_depth(3) // Only search up to 3 levels deep
        .build();

    let results = PatternHunt::sync(
        &["**/*.rs"],
        &["."],
        options
    )?;

    println!("Found {} files with depth limit", results.len());
    Ok(())
}
```

### Case-Insensitive Search

```rust
use patternhunt::{PatternHunt, GlobOptions, GlobOptionsBuilder};

fn case_insensitive() -> Result<(), Box<dyn std::error::Error>> {
    // Case-insensitive search (useful on case-sensitive filesystems)
    let options = GlobOptionsBuilder::new()
        .case_sensitive(false)
        .build();

    let results = PatternHunt::sync(
        &["**/*.RS", "**/*.MD"], // Upper-case patterns
        &["."],
        options
    )?;

    println!("Found {} files with case-insensitive search", results.len());
    Ok(())
}
```

## Modules

- **`brace.rs`**: Handles brace expansion with support for nested braces and numeric ranges, with protections against excessive recursion and expansion counts.
- **`cache.rs`**: Implements LRU caching for compiled glob patterns and regexes, with TTL-based expiration and performance metrics.
- **`micromatch.rs`**: Converts extended glob patterns to regex, supporting features like character classes and extglobs.
- **`mod.rs`**: Core pattern compilation and matching logic, integrating brace expansion, regex, and glob patterns.
- **`batch_io.rs`**: Provides efficient filesystem metadata access with caching and symlink handling.
- **`error.rs`**: Defines comprehensive error types for all glob operations.
- **`async_glob.rs`**: Implements asynchronous globbing with a streaming API and bounded concurrency.
- **`options.rs`**: Configures globbing behavior with a builder pattern for flexible customization.
- **`predicates.rs`**: Filters files based on metadata attributes like size, type, and timestamps.
- **`sync.rs`**: Implements synchronous globbing using `WalkDir` for efficient directory traversal.

## Error Handling

The library uses a comprehensive `GlobError` enum to handle errors, including:

- I/O errors (`Io`)
- Regex compilation errors (`Regex`)
- Invalid pattern syntax (`InvalidPattern`)
- Path traversal attempts (`PathTraversal`)
- Symlink cycles (`SymlinkCycle`)
- Permission issues (`PermissionDenied`)
- Excessive brace expansion (`BraceExpansionDepth`, `BraceExpansionCount`)
- Regex complexity limits (`RegexTooComplex`)

## Performance Considerations

- **Caching**: Use `cache_metrics()` to monitor cache performance and adjust `MAX_CACHE_SIZE` or TTL as needed.
- **Concurrency**: For async operations, tune `max_inflight` in `GlobOptions` to balance performance and resource usage.
- **Depth Limits**: Set `max_depth` to avoid excessive traversal in deep directory structures.
- **Symlink Handling**: Disable `follow_symlinks` if symlinks are not needed to reduce I/O overhead.

## Security Considerations

- **Path Traversal Protection**: Patterns containing `../` are rejected to prevent unauthorized access.
- **Symlink Cycle Detection**: Prevents infinite loops when following symlinks.
- **Resource Limits**: Caps on brace expansion and regex complexity prevent resource exhaustion.
- **Permission Checks**: Ensures files are readable before processing.

## Testing

The library includes comprehensive unit tests for each module, covering:

- Brace expansion (`brace.rs`)
- Cache performance and eviction (`cache.rs`)
- Extended glob pattern conversion (`micromatch.rs`)
- Pattern compilation and matching (`mod.rs`)

Run tests with:

```bash
cargo test
```

## Contributing

Contributions are welcome! Please submit issues or pull requests to the GitHub repository. Ensure code follows Rust conventions and includes tests for new features.

## License

This project is licensed under the MIT License. See the `LICENSE` file for details.
