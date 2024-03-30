use crate::Key;
use std::path::PathBuf;

/// Individual file information
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// Key Information
    pub key: Key,
    /// File Path
    pub path: PathBuf,
}
