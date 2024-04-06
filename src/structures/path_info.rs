use serde::{
    Deserialize,
    Serialize, //Serializer,
};
use std::path::PathBuf;

/// File Information including path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathInfo {
    /// File size (in bytes)
    #[serde(rename = "File size (bytes)")]
    pub size: usize,
    /// Hash
    #[serde(rename = "Hash")] // serialize_with = "add_quotes"
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

/*
/// Add quotes
fn add_quotes<S>(hash: &Option<String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match hash {
        Some(string) => match string.parse::<u128>() {
            Ok(integer) if integer > 1_000_000_000_000 => {
                serializer.collect_str(&format!("'{:?}'", integer))
            }
            Ok(_) => serializer.collect_str(&string.to_string()),
            Err(_) => serializer.collect_str(&string.to_string()),
        },
        None => serializer.serialize_none(),
    }
}
*/
