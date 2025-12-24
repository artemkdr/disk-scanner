# Copilot Instructions for disk-scanner

## Project Overview

**disk-scanner** is a fast, cross-platform CLI tool for analyzing disk usage written in Rust. It scans directories and identifies the largest files and folders, displaying them sorted by size in descending order.

### Key Features

- Parallel directory traversal using `jwalk` and `rayon`
- Cross-platform file size calculation with `filesize` crate
- Beautiful terminal output with colors and progress indication
- Configurable depth and item count

## Architecture

```
src/
├── main.rs       # Entry point, orchestrates CLI → Scanner → Display
├── cli.rs        # Command-line argument parsing (clap derive)
├── scanner.rs    # Parallel directory traversal and size calculation
├── node.rs       # Data structures (Node, ScanResult)
└── display.rs    # Output formatting and rendering
```

### Module Responsibilities

- **cli.rs**: Defines `Args` struct with clap derive macros. All CLI configuration lives here.
- **scanner.rs**: Contains `Scanner` struct with builder pattern. Handles parallel traversal, file size calculation, and progress reporting.
- **node.rs**: Defines `Node` (single entry) and `ScanResult` (collection with stats). Pure data structures with filtering/sorting methods.
- **display.rs**: Formats and prints results. Supports colored output with `owo-colors` and human-readable sizes with `humansize`.

## Dependencies Rationale

| Crate | Purpose | Why Chosen |
|-------|---------|------------|
| `clap` | CLI parsing | Industry standard, derive macros reduce boilerplate |
| `jwalk` | Directory walking | Parallel traversal, faster than walkdir for large directories |
| `rayon` | Parallelism | Work-stealing, used by jwalk internally |
| `filesize` | Disk usage | Cross-platform, handles NTFS compression, sparse files |
| `anyhow` | Error handling | Ergonomic for CLI apps, good context chaining |
| `indicatif` | Progress bars | Beautiful spinners, multi-progress support |
| `owo-colors` | Terminal colors | Zero-allocation, works on all platforms |
| `humansize` | Size formatting | Configurable (binary/decimal), well-maintained |

## Coding Conventions

### Error Handling

- Use `anyhow::Result` for all fallible functions
- Add context with `.with_context(|| format!(...))` for user-facing errors
- Propagate errors with `?` operator, don't unwrap in library code
- Handle permission errors gracefully (count them, don't crash)

```rust
// Good
fn scan(path: &Path) -> Result<ScanResult> {
    let path = path.canonicalize()
        .with_context(|| format!("Failed to resolve: {}", path.display()))?;
    // ...
}

// Avoid
fn scan(path: &Path) -> ScanResult {
    let path = path.canonicalize().unwrap(); // Don't do this
}
```

### Builder Pattern

Use builder pattern for configurable types:

```rust
let scanner = Scanner::new()
    .with_threads(Some(4))
    .include_files(true);
```

### Struct Organization

```rust
pub struct MyStruct {
    // Public fields first (if any)
    pub public_field: Type,
    
    // Private fields
    private_field: Type,
}

impl Default for MyStruct { ... }

impl MyStruct {
    pub fn new() -> Self { ... }
    
    // Builder methods
    pub fn with_option(mut self, opt: Type) -> Self { ... }
    
    // Public methods
    pub fn do_something(&self) -> Result<()> { ... }
}

// Private helper functions at module level
fn helper_function() { ... }
```

### Testing

- Unit tests go in `#[cfg(test)]` module at bottom of each file
- Integration tests in `tests/` directory
- Use `tempfile` for filesystem tests
- Use `assert_cmd` for CLI tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_specific_behavior() {
        // Arrange
        let input = ...;
        
        // Act
        let result = function(input);
        
        // Assert
        assert_eq!(result, expected);
    }
}
```

## Commands

### Build

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release
```

### Run

```bash
# Scan current directory
cargo run

# Scan specific path with options
cargo run -- /path/to/scan -n 20 --all

# Run release build
cargo run --release -- /path/to/scan
```

### Test

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

### Lint & Format

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Run linter
cargo clippy

# Run linter with stricter checks
cargo clippy -- -W clippy::pedantic
```

## Future Enhancements

When adding features, consider:

1. **TUI Mode**: Add optional interactive mode with `ratatui`
2. **Output Formats**: Add `--json` and `--csv` flags
3. **Ignore Patterns**: Add `--exclude` flag with glob patterns
4. **Config File**: Support `~/.config/disk-scanner/config.toml`
5. **Shell Completions**: Generate with `clap_complete` in build.rs

## Cross-Platform Notes

### Windows

- Use `filesize::PathExt::size_on_disk()` for accurate NTFS sizes
- Handle long paths (>260 chars) - Rust handles this automatically
- Some system directories (System Volume Information) require admin

### Unix/macOS

- Use `metadata.blocks() * 512` for actual disk usage
- Handle hard links to avoid double-counting
- Skip virtual filesystems: /proc, /sys, /dev

### Code Pattern

```rust
#[cfg(windows)]
fn platform_specific() {
    // Windows implementation
}

#[cfg(unix)]
fn platform_specific() {
    // Unix implementation
}
```
