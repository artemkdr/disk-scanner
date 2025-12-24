//! Data structures representing file system entries with their sizes.

use std::path::PathBuf;

/// Represents a file system entry (file or directory) with its size.
#[derive(Debug, Clone)]
pub struct Node {
    /// Absolute path to the entry
    pub path: PathBuf,
    /// Size in bytes
    pub size: u64,
    /// Whether this is a directory
    pub is_dir: bool,
    /// Depth relative to the scan root
    pub depth: usize,
}

impl Node {
    /// Create a new Node
    pub fn new(path: PathBuf, size: u64, is_dir: bool, depth: usize) -> Self {
        Self {
            path,
            size,
            is_dir,
            depth,
        }
    }
}

/// Collection of nodes with aggregate statistics
#[derive(Debug, Default)]
pub struct ScanResult {
    /// All scanned entries
    pub nodes: Vec<Node>,
    /// Total size of all files scanned
    pub total_size: u64,
    /// Total number of files scanned
    pub file_count: u64,
    /// Total number of directories scanned
    pub dir_count: u64,
    /// Number of errors encountered
    pub error_count: u64,
}

impl ScanResult {
    /// Create a new empty ScanResult
    pub fn new() -> Self {
        Self::default()
    }

    /// Sort nodes by size in descending order
    pub fn sort_by_size_desc(&mut self) {
        self.nodes.sort_by(|a, b| b.size.cmp(&a.size));
    }

    /// Get the top N nodes by size
    pub fn top_n(&self, n: usize) -> &[Node] {
        let end = std::cmp::min(n, self.nodes.len());
        &self.nodes[..end]
    }

    /// Filter nodes by maximum depth
    pub fn filter_by_depth(&mut self, max_depth: usize) {
        self.nodes.retain(|node| node.depth <= max_depth);
    }

    /// Filter to only include directories
    pub fn filter_dirs_only(&mut self) {
        self.nodes.retain(|node| node.is_dir);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_by_size() {
        let mut result = ScanResult::new();
        result
            .nodes
            .push(Node::new(PathBuf::from("small"), 100, false, 1));
        result
            .nodes
            .push(Node::new(PathBuf::from("large"), 1000, false, 1));
        result
            .nodes
            .push(Node::new(PathBuf::from("medium"), 500, false, 1));

        result.sort_by_size_desc();

        assert_eq!(result.nodes[0].size, 1000);
        assert_eq!(result.nodes[1].size, 500);
        assert_eq!(result.nodes[2].size, 100);
    }

    #[test]
    fn test_top_n() {
        let mut result = ScanResult::new();
        for i in 0..20 {
            result.nodes.push(Node::new(
                PathBuf::from(format!("file{}", i)),
                i as u64 * 100,
                false,
                1,
            ));
        }
        result.sort_by_size_desc();

        let top5 = result.top_n(5);
        assert_eq!(top5.len(), 5);
        assert_eq!(top5[0].size, 1900);
    }
}
