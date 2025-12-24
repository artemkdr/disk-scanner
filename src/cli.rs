//! Command-line argument parsing using clap derive macros.

use clap::Parser;
use std::path::PathBuf;

/// A fast, cross-platform CLI tool for analyzing disk usage.
///
/// Scans directories and displays the largest files and folders,
/// sorted by size in descending order.
#[derive(Parser, Debug)]
#[command(name = "disk-scanner")]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Target directory to scan (defaults to current directory)
    #[arg(value_name = "PATH", default_value = ".")]
    pub path: PathBuf,

    /// Number of top items to display
    #[arg(short = 'n', long = "count", default_value = "10")]
    pub count: usize,

    /// Maximum depth to display (unlimited if not specified)
    #[arg(short = 'd', long = "depth")]
    pub depth: Option<usize>,

    /// Show files in addition to directories
    #[arg(short, long)]
    pub all: bool,

    /// Number of threads to use (defaults to number of CPU cores)
    #[arg(short = 't', long = "threads")]
    pub threads: Option<usize>,
}

impl Args {
    /// Parse command-line arguments
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_args() {
        let args = Args::parse_from(["disk-scanner"]);
        assert_eq!(args.path, PathBuf::from("."));
        assert_eq!(args.count, 10);
        assert_eq!(args.depth, None);
        assert_eq!(args.threads, None);
        assert!(!args.all);
    }

    #[test]
    fn test_custom_args() {
        let args = Args::parse_from([
            "disk-scanner",
            "/some/path",
            "-n",
            "20",
            "-d",
            "3",
            "--all",
            "-t",
            "4",
        ]);
        assert_eq!(args.path, PathBuf::from("/some/path"));
        assert_eq!(args.count, 20);
        assert_eq!(args.depth, Some(3));
        assert!(args.all);
        assert_eq!(args.threads, Some(4));
    }
}
