use serde::Serialize;
use std::path::PathBuf;

/// File Information including path
#[derive(Debug, Clone, Serialize)]
pub struct PathInfo {
    /// File size (in bytes)
    #[serde(rename = "File size (bytes)")]
    pub size: usize,
    /// Hash
    #[serde(rename = "Hash")]
    pub hash: Option<String>,
    /// File Paths
    #[serde(rename = "Path")]
    pub path: PathBuf,
    /// Number of duplicate files with the same size and blake3 hash
    #[serde(rename = "Number of duplicate files")]
    pub num_file: usize,
    /// Sum of individual file sizes declared in paths
    #[serde(rename = "Sum of file sizes (bytes)")]
    pub sum_size: usize,
}
