//! Directory scanning logic using parallel traversal.

use crate::node::{Node, ScanResult};
use anyhow::{Context, Result};
use filesize::PathExt;
use indicatif::{ProgressBar, ProgressStyle};
use jwalk::WalkDir;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Scanner configuration
#[derive(Default)]
pub struct Scanner {
    /// Number of threads to use (None = use all cores)
    pub num_threads: Option<usize>,
    /// Whether to include files in results (not just directories)
    pub include_files: bool,
}

/// Entry collected during scanning
struct ScannedEntry {
    path: PathBuf,
    size: u64,
    is_dir: bool,
    depth: usize,
}

impl Scanner {
    /// Create a new Scanner with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the number of threads
    pub fn with_threads(mut self, threads: Option<usize>) -> Self {
        self.num_threads = threads;
        self
    }

    /// Include files in the results
    pub fn include_files(mut self, include: bool) -> Self {
        self.include_files = include;
        self
    }

    /// Scan a directory and return results
    pub fn scan(&self, root: &Path) -> Result<ScanResult> {
        let root = root
            .canonicalize()
            .with_context(|| format!("Failed to resolve path: {}", root.display()))?;

        // Setup progress indicator
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .expect("Invalid progress template"),
        );
        pb.enable_steady_tick(Duration::from_millis(100));
        pb.set_message("Starting scan...");

        // Track total scan duration
        let start_time = Instant::now();

        // Shared state for progress tracking
        let files_scanned = Arc::new(AtomicU64::new(0));
        let dirs_scanned = Arc::new(AtomicU64::new(0));
        let total_size = Arc::new(AtomicU64::new(0));
        let error_count = Arc::new(AtomicU64::new(0));
        let last_update = Arc::new(Mutex::new(Instant::now()));
        let current_dir = Arc::new(Mutex::new(String::from("...")));

        // Collected entries (always collect files for size calculation)
        let entries: Arc<Mutex<Vec<ScannedEntry>>> = Arc::new(Mutex::new(Vec::new()));

        // Configure walker
        let num_threads = self.num_threads.unwrap_or_else(num_cpus);
        let walker = WalkDir::new(&root)
            .parallelism(jwalk::Parallelism::RayonNewPool(num_threads))
            .skip_hidden(false)
            .follow_links(false);

        // Clone references for the closure
        let files_scanned_clone = Arc::clone(&files_scanned);
        let dirs_scanned_clone = Arc::clone(&dirs_scanned);
        let total_size_clone = Arc::clone(&total_size);
        let error_count_clone = Arc::clone(&error_count);
        let last_update_clone = Arc::clone(&last_update);
        let current_dir_clone = Arc::clone(&current_dir);
        let entries_clone = Arc::clone(&entries);
        let pb_clone = pb.clone();

        // Process entries in parallel - calculate sizes during walk
        walker.into_iter().for_each(|entry_result| {
            match entry_result {
                Ok(entry) => {
                    let path = entry.path();
                    let is_dir = entry.file_type().is_dir();
                    let depth = entry.depth();

                    if is_dir {
                        dirs_scanned_clone.fetch_add(1, Ordering::Relaxed);

                        // Update current directory being scanned
                        if let Ok(mut dir) = current_dir_clone.try_lock() {
                            if let Some(name) = path.file_name() {
                                *dir = name.to_string_lossy().to_string();
                            }
                        }

                        // Add directory entry (size will be calculated later)
                        if depth > 0 {
                            if let Ok(mut entries) = entries_clone.try_lock() {
                                entries.push(ScannedEntry {
                                    path: path.clone(),
                                    size: 0,
                                    is_dir: true,
                                    depth,
                                });
                            }
                        }
                    } else {
                        files_scanned_clone.fetch_add(1, Ordering::Relaxed);

                        // Get file size immediately
                        let size = get_file_size(&path).unwrap_or(0);
                        total_size_clone.fetch_add(size, Ordering::Relaxed);

                        // Always add file entry (needed for directory size calculation)
                        if let Ok(mut entries) = entries_clone.try_lock() {
                            entries.push(ScannedEntry {
                                path: path.clone(),
                                size,
                                is_dir: false,
                                depth,
                            });
                        }
                    }

                    // Update progress display (throttled to avoid flickering)
                    if let Ok(mut last) = last_update_clone.try_lock() {
                        if last.elapsed() >= Duration::from_millis(50) {
                            *last = Instant::now();
                            let files = files_scanned_clone.load(Ordering::Relaxed);
                            let dirs = dirs_scanned_clone.load(Ordering::Relaxed);
                            let size = total_size_clone.load(Ordering::Relaxed);
                            let dir_name = current_dir_clone
                                .lock()
                                .map(|d| d.clone())
                                .unwrap_or_default();

                            pb_clone.set_message(format!(
                                "Scanning: {} | {} files, {} dirs | {}",
                                truncate_str(&dir_name, 20),
                                format_number(files),
                                format_number(dirs),
                                format_size_simple(size)
                            ));
                        }
                    }
                }
                Err(_) => {
                    error_count_clone.fetch_add(1, Ordering::Relaxed);
                }
            }
        });

        let file_count = files_scanned.load(Ordering::Relaxed);
        let dir_count = dirs_scanned.load(Ordering::Relaxed);
        let scanned_size = total_size.load(Ordering::Relaxed);

        pb.set_message(format!(
            "Calculating directory sizes... ({} files, {})",
            format_number(file_count),
            format_size_simple(scanned_size)
        ));

        // Now calculate directory sizes by aggregating from entries
        let mut dir_sizes: HashMap<PathBuf, u64> = HashMap::new();
        let all_entries = entries.lock().unwrap();

        // Initialize all directories
        for entry in all_entries.iter() {
            if entry.is_dir {
                dir_sizes.insert(entry.path.clone(), 0);
            }
        }

        // Add file sizes to parent directories
        let file_entries: Vec<_> = all_entries.iter().filter(|e| !e.is_dir).collect();
        let total_files = file_entries.len();

        for (idx, entry) in file_entries.iter().enumerate() {
            // Update progress for directory calculation
            if idx % 50000 == 0 && total_files > 0 {
                pb.set_message(format!(
                    "Aggregating sizes... {:.0}%",
                    (idx as f64 / total_files as f64) * 100.0
                ));
            }

            // Propagate size up to all parent directories
            let mut current = entry.path.parent();
            while let Some(parent) = current {
                if let Some(dir_size) = dir_sizes.get_mut(parent) {
                    *dir_size += entry.size;
                }
                if parent == root {
                    break;
                }
                current = parent.parent();
            }
        }

        pb.set_message("Building results...");

        // Build the result
        let mut result = ScanResult::new();
        result.file_count = file_count;
        result.dir_count = dir_count.saturating_sub(1); // Exclude root
        result.total_size = scanned_size;
        result.error_count = error_count.load(Ordering::Relaxed);

        // Add directories with their calculated sizes
        for (path, size) in dir_sizes {
            let depth = path
                .strip_prefix(&root)
                .map(|p| p.components().count())
                .unwrap_or(0);
            result.nodes.push(Node::new(path, size, true, depth));
        }

        // Add files if requested
        if self.include_files {
            for entry in all_entries.iter() {
                if !entry.is_dir {
                    result.nodes.push(Node::new(
                        entry.path.clone(),
                        entry.size,
                        false,
                        entry.depth,
                    ));
                }
            }
        }

        let duration = start_time.elapsed();
        pb.finish_with_message(format!(
            "Done! {} files, {} dirs ({}) in {}",
            format_number(result.file_count),
            format_number(result.dir_count),
            format_size_simple(result.total_size),
            format_duration(duration)
        ));

        Ok(result)
    }
}

/// Get the size of a file on disk
fn get_file_size(path: &Path) -> Option<u64> {
    path.size_on_disk()
        .ok()
        .or_else(|| path.metadata().ok().map(|m| m.len()))
}

/// Get the number of CPU cores
fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

/// Format a number with thousand separators
fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

/// Truncate a string to max length with ellipsis
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Format a duration in human-readable form
fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();

    if secs >= 60 {
        let mins = secs / 60;
        let remaining_secs = secs % 60;
        format!("{}m {}s", mins, remaining_secs)
    } else if secs > 0 {
        format!("{}.{:02}s", secs, millis / 10)
    } else {
        format!("{}ms", millis)
    }
}

/// Simple size formatting for progress messages
fn format_size_simple(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_scan_empty_dir() {
        let dir = tempdir().unwrap();
        let scanner = Scanner::new();
        let result = scanner.scan(dir.path()).unwrap();
        assert_eq!(result.file_count, 0);
    }

    #[test]
    fn test_scan_with_files() {
        let dir = tempdir().unwrap();

        fs::write(dir.path().join("small.txt"), "hello").unwrap();
        fs::write(dir.path().join("large.txt"), "x".repeat(1000)).unwrap();
        fs::create_dir(dir.path().join("subdir")).unwrap();
        fs::write(dir.path().join("subdir/nested.txt"), "nested content").unwrap();

        let scanner = Scanner::new().include_files(true);
        let result = scanner.scan(dir.path()).unwrap();

        assert_eq!(result.file_count, 3);
        assert!(result.total_size > 0);
    }

    #[test]
    fn test_format_size_simple() {
        assert_eq!(format_size_simple(500), "500 B");
        assert_eq!(format_size_simple(1024), "1.0 KB");
        assert_eq!(format_size_simple(1536), "1.5 KB");
        assert_eq!(format_size_simple(1048576), "1.0 MB");
        assert_eq!(format_size_simple(1073741824), "1.00 GB");
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(999), "999");
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(1234567), "1,234,567");
    }

    #[test]
    fn test_truncate_str() {
        assert_eq!(truncate_str("short", 10), "short");
        assert_eq!(truncate_str("this is a long string", 10), "this is...");
    }
}
