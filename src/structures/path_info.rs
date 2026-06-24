use crate::excel::{fmt_center, fmt_default, fmt_integer};
use rust_xlsxwriter::XlsxSerialize;
use serde::Serialize;
use std::path::PathBuf;

/// Detailed information about a specific file path within an identical group.
///
/// This struct is optimized for serialization into CSV and XLSX formats.
/// It uses `rust_xlsxwriter` attributes to define the visual style of the Excel report.
#[derive(XlsxSerialize, Serialize, Debug, Clone)]
#[xlsx(table_default)]
pub struct PathInfo {
    /// Individual file size in bytes.
    #[serde(rename = "File size (bytes)")]
    #[xlsx(value_format = fmt_integer())]
    pub size: usize,

    /// Calculated hash (Blake3 or other) used to verify identity.
    #[serde(rename = "Hash")]
    #[xlsx(value_format = fmt_center())]
    pub hash: Option<String>,

    /// The absolute or relative path to the file.
    #[serde(rename = "Path")]
    #[xlsx(value_format = fmt_default())]
    pub path: PathBuf,

    /// How many files were found with this exact size and hash.
    #[serde(rename = "Frequency")]
    #[xlsx(value_format = fmt_integer())]
    pub num_file: usize,

    /// The total storage wasted by all files in this specific identical group.
    #[serde(rename = "Sum of file sizes (bytes)")]
    #[xlsx(value_format = fmt_integer())]
    pub sum_size: usize,
}
