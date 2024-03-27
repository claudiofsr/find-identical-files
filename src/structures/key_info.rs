use serde::Serialize;
use crate::add_thousands_separator;

/// This key will be used by FileInfo and GroupInfo.
///
/// For the FileInfo struct, the hash will be None.
///
/// For the GroupInfo struct, the hash will be Some(blake3).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct Key {
    /// Individual file size (in bytes)
    #[serde(serialize_with = "add_thousands_separator")]
    pub size: usize,
    /// Blake3 hash
    pub hash: Option<String>,
}
