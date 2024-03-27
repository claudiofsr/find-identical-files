use std::path::PathBuf;
use crate::Key;

/// Individual file information
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// Key Information
    pub key: Key,
    /// File Path
    pub path: PathBuf,
}
