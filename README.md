# disk-scanner

A fast, cross-platform CLI tool for analyzing disk usage written in Rust. Scans directories and identifies the largest files and folders, displaying them sorted by size in descending order.

## Features

- **Parallel directory traversal** - Uses `jwalk` and `rayon` for efficient multi-threaded scanning
- **Cross-platform** - Works on Linux, macOS, and Windows
- **Accurate disk usage** - Properly handles NTFS compression, sparse files, and platform differences
- **Beautiful output** - Colored terminal output with human-readable file sizes
- **Configurable** - Customize depth, item count, and filtering options
- **Progress indication** - Real-time progress bars during scanning

## Installation

### Build from source

```bash
git clone https://github.com/artemkdr/disk-scanner.git
cd disk-scanner
cargo build --release
```

The binary will be available at `target/release/disk-scanner`.

## Usage

### Basic usage

Scan the current directory:

```bash
disk-scanner
```

Scan a specific path:

```bash
disk-scanner /path/to/scan
```

### Options

- `-n, --number <N>` - Number of items to display (default: 10)
- `-d, --depth <DEPTH>` - Maximum directory depth to scan
- `-a, --all` - Include hidden files and directories
- `-t, --threads <N>` - Number of threads to use (default: number of CPU cores)

### Examples

```bash
# Show top 20 largest items in /var
disk-scanner /var -n 20

# Scan with specific thread count
disk-scanner . -t 4

# Include hidden files
disk-scanner . --all

# Combine options
disk-scanner /home -n 50 -d 3 --all
```

## Building

### Debug build

```bash
cargo build
```

### Release build (optimized)

```bash
cargo build --release
```

## Testing

Run all tests:

```bash
cargo test
```

Run tests with output:

```bash
cargo test -- --nocapture
```

Run a specific test:

```bash
cargo test test_name
```

## Development

### Code style

Format code:

```bash
cargo fmt
```

Check formatting:

```bash
cargo fmt --check
```

### Linting

Run clippy:

```bash
cargo clippy
```

Run with stricter checks:

```bash
cargo clippy -- -W clippy::pedantic
```

## Project Structure

```
src/
├── main.rs       # Entry point, orchestrates CLI → Scanner → Display
├── cli.rs        # Command-line argument parsing (clap derive)
├── scanner.rs    # Parallel directory traversal and size calculation
├── node.rs       # Data structures (Node, ScanResult)
└── display.rs    # Output formatting and rendering
```

### Module Overview

- **cli.rs** - Defines `Args` struct with clap derive macros. All CLI configuration lives here.
- **scanner.rs** - Contains `Scanner` struct with builder pattern. Handles parallel traversal, file size calculation, and progress reporting.
- **node.rs** - Defines `Node` (single entry) and `ScanResult` (collection with stats). Pure data structures with filtering/sorting methods.
- **display.rs** - Formats and prints results. Supports colored output with `owo-colors` and human-readable sizes with `humansize`.

## Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` | CLI parsing with derive macros |
| `jwalk` | Parallel directory traversal |
| `rayon` | Parallelism and work-stealing |
| `filesize` | Cross-platform disk usage calculation |
| `anyhow` | Ergonomic error handling |
| `indicatif` | Progress bars and spinners |
| `owo-colors` | Terminal colors (zero-allocation) |
| `humansize` | Human-readable size formatting |

## Platform-Specific Notes

### Windows

- Uses `filesize::PathExt::size_on_disk()` for accurate NTFS sizes
- Handles NTFS compression and alternate data streams
- Supports long paths (>260 characters)

### Linux/macOS

- Uses `metadata.blocks() * 512` for actual disk usage
- Properly handles hard links
- Skips virtual filesystems (/proc, /sys, /dev)

## CI/CD

The project uses GitHub Actions for continuous integration:

- **Runs on**: Linux (Ubuntu), Windows, macOS
- **Tests**: Format check, clippy linting, unit tests
- **Caching**: Cargo dependencies are cached for faster builds

## Performance

disk-scanner is optimized for speed:

- Parallel directory traversal using rayon work-stealing scheduler
- Minimal allocations using zero-cost abstractions
- Efficient file metadata reading
- Progress reporting without blocking the scan

## License

MIT

## Contributing

Contributions are welcome! Please ensure:

1. Code passes `cargo fmt` and `cargo clippy`
2. All tests pass: `cargo test`
3. Code is documented with comments for complex logic

## Future Enhancements

Potential features for future versions:

- Interactive TUI mode with `ratatui`
- JSON and CSV output formats
- Exclude patterns with glob matching
- Config file support (`~/.config/disk-scanner/config.toml`)
- Shell completions generation
