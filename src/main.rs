//! disk-scanner: A fast, cross-platform CLI tool for analyzing disk usage.
//!
//! This tool scans directories and identifies the largest files and folders,
//! displaying them sorted by size in descending order.

mod cli;
mod display;
mod node;
mod scanner;

use anyhow::{Context, Result};
use cli::Args;
use display::Display;
use scanner::Scanner;

fn main() -> Result<()> {
    let args = Args::parse_args();

    // Validate the path exists
    let path = args.path.canonicalize().with_context(|| {
        format!(
            "Cannot access path '{}': No such file or directory",
            args.path.display()
        )
    })?;

    if !path.is_dir() {
        anyhow::bail!("'{}' is not a directory", path.display());
    }

    // Configure and run the scanner
    let scanner = Scanner::new()
        .with_threads(args.threads)
        .include_files(args.all);

    let mut result = scanner
        .scan(&path)
        .with_context(|| format!("Failed to scan '{}'", path.display()))?;

    // Apply filters
    if !args.all {
        result.filter_dirs_only();
    }

    if let Some(depth) = args.depth {
        result.filter_by_depth(depth);
    }

    // Sort by size descending
    result.sort_by_size_desc();

    // Display results
    let display = Display::new().with_count(args.count);
    display.print_results(&result, &path);

    Ok(())
}
