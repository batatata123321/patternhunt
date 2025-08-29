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
glob-lib = { git = "https://github.com/username/glob-lib.git" }
```

Ensure you have the required dependencies installed, including `globset`, `regex`, `lru`, `once_cell`, `camino`, `walkdir`, and `tokio` (for async features).

## Usage

### Basic Example (Synchronous)

```rust
use glob_lib::{GlobOptions, sync::glob_sync, Patterns};
use camino::Utf8PathBuf;

fn main() -> Result<(), glob_lib::GlobError> {
    let patterns = Patterns::compile_many(&["*.txt", "src/*.rs"], &GlobOptions::default())?;
    let results = glob_sync(patterns, GlobOptions::default(), None)?;

    for path in results {
        println!("Found: {}", path.display());
    }
    Ok(())
}
```

### Asynchronous Example

```rust
use glob_lib::{GlobOptions, async_glob::glob_stream, Patterns};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), glob_lib::GlobError> {
    let patterns = Patterns::compile_many(&["*.txt", "src/*.rs"], &GlobOptions::default())?;
    let mut stream = glob_stream(patterns, GlobOptions::default(), None);

    while let Some(result) = stream.next().await {
        match result {
            Ok(path) => println!("Found: {}", path.display()),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
    Ok(())
}
```

### Advanced Example with Predicates

```rust
use glob_lib::{GlobOptions, GlobOptionsBuilder, Predicates, FileType, sync::glob_sync, Patterns};
use std::time::{Duration, SystemTime};

fn main() -> Result<(), glob_lib::GlobError> {
    // Configure predicates to filter files
    let predicates = Predicates {
        min_size: Some(1024), // Minimum 1KB
        max_size: Some(1024 * 1024), // Maximum 1MB
        file_type: Some(FileType::File),
        mtime_after: Some(SystemTime::now() - Duration::from_secs(24 * 60 * 60)),
        ..Default::default()
    };

    // Configure glob options
    let opts = GlobOptionsBuilder::new()
        .follow_symlinks(true)
        .max_depth(3)
        .case_sensitive(true)
        .predicates(predicates)
        .build();

    // Compile patterns
    let patterns = Patterns::compile_many(&["src/**/*.{rs,toml}"], &opts)?;

    // Perform synchronous globbing
    let results = glob_sync(patterns, opts, None)?;

    for path in results {
        println!("Found: {}", path.display());
    }
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
