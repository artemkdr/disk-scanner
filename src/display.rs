//! Output formatting and display logic.

use crate::node::{Node, ScanResult};
use humansize::{BINARY, format_size};
use owo_colors::OwoColorize;

/// Display configuration
pub struct Display {
    /// Maximum number of items to show
    pub count: usize,
    /// Maximum path width before truncation
    pub max_path_width: usize,
}

impl Default for Display {
    fn default() -> Self {
        Self {
            count: 10,
            max_path_width: 60,
        }
    }
}

impl Display {
    /// Create a new Display with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the number of items to display
    pub fn with_count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }

    /// Print the scan results to stdout
    pub fn print_results(&self, result: &ScanResult, root_path: &std::path::Path) {
        println!();
        println!("{}", "═".repeat(70).dimmed());
        println!(
            "{}",
            format!(" Disk Usage Report: {}", root_path.display()).bold()
        );
        println!("{}", "═".repeat(70).dimmed());
        println!();

        // Print summary
        println!(
            "  {} {}",
            "Total size:".dimmed(),
            format_size(result.total_size, BINARY).green().bold()
        );
        println!(
            "  {} {} files, {} directories",
            "Scanned:".dimmed(),
            result.file_count.to_string().cyan(),
            result.dir_count.to_string().cyan()
        );

        if result.error_count > 0 {
            println!(
                "  {} {} (permission denied or inaccessible)",
                "Errors:".dimmed(),
                result.error_count.to_string().red()
            );
        }

        println!();
        println!("{}", "─".repeat(70).dimmed());
        println!("{}", format!(" Top {} by size:", self.count).bold());
        println!("{}", "─".repeat(70).dimmed());
        println!();

        // Print header
        println!(
            "  {:>12}  {}",
            "SIZE".dimmed().bold(),
            "PATH".dimmed().bold()
        );
        println!("  {:>12}  {}", "────".dimmed(), "────".dimmed());

        // Print top entries
        let top_nodes = result.top_n(self.count);

        if top_nodes.is_empty() {
            println!("  {}", "No entries found.".dimmed());
        } else {
            for node in top_nodes {
                self.print_node(node, root_path);
            }
        }

        println!();
        println!("{}", "═".repeat(70).dimmed());
    }

    /// Print a single node
    fn print_node(&self, node: &Node, root_path: &std::path::Path) {
        let size_str = format_size(node.size, BINARY);
        let relative_path = node.path.strip_prefix(root_path).unwrap_or(&node.path);

        let path_str = relative_path.display().to_string();
        let display_path = self.truncate_path(&path_str);

        let (icon, styled_path) = if node.is_dir {
            ("📁", display_path.blue().bold().to_string())
        } else {
            ("📄", display_path.white().to_string())
        };

        // Calculate percentage of total if we had access to it
        println!("  {:>12}  {} {}", size_str.green(), icon, styled_path);
    }

    /// Truncate a path if it's too long
    fn truncate_path(&self, path: &str) -> String {
        if path.len() <= self.max_path_width {
            path.to_string()
        } else {
            let start = path.len() - self.max_path_width + 3;
            format!("...{}", &path[start..])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_path_short() {
        let display = Display::new();
        let path = "short/path";
        assert_eq!(display.truncate_path(path), "short/path");
    }

    #[test]
    fn test_truncate_path_long() {
        let display = Display {
            max_path_width: 20,
            ..Default::default()
        };
        let path = "this/is/a/very/long/path/that/should/be/truncated";
        let truncated = display.truncate_path(path);
        assert!(truncated.starts_with("..."));
        assert!(truncated.len() <= 23); // 20 + "..."
    }
}
