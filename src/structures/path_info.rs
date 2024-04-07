use crate::excel::{FONT_NAME, FONT_SIZE};
use rust_xlsxwriter::XlsxSerialize;
use serde::{
    Serialize,
    //Serializer,
};
use std::path::PathBuf;

/// File Information including path
///
/// rust_xlsxwriter: Working with Serde
///
/// <https://docs.rs/rust_xlsxwriter/latest/rust_xlsxwriter/serializer/index.html>
#[derive(XlsxSerialize, Serialize)]
#[xlsx(table_default)]
pub struct PathInfo {
    /// File size (in bytes)
    #[serde(rename = "File size (bytes)")]
    #[xlsx(
        value_format = Format::new()
            .set_align(FormatAlign::VerticalCenter)
            .set_num_format("#,##0")
            .set_font_name(FONT_NAME)
            .set_font_size(FONT_SIZE)
    )]
    pub size: usize,

    /// Hash
    #[serde(rename = "Hash")] // serialize_with = "add_quotes"
    #[xlsx(
        value_format = Format::new()
            .set_align(FormatAlign::Center)
            .set_align(FormatAlign::VerticalCenter)
            .set_font_name(FONT_NAME)
            .set_font_size(FONT_SIZE)
    )]
    pub hash: Option<String>,

    /// File Paths
    #[serde(rename = "Path")]
    #[xlsx(
        value_format = Format::new()
            .set_align(FormatAlign::VerticalCenter)
            .set_font_name(FONT_NAME)
            .set_font_size(FONT_SIZE)
    )]
    pub path: PathBuf,

    /// Number of duplicate files with the same size and blake3 hash
    #[serde(rename = "Number of duplicate files")]
    #[xlsx(
        value_format = Format::new()
            .set_align(FormatAlign::VerticalCenter)
            .set_num_format("#,##0")
            .set_font_name(FONT_NAME)
            .set_font_size(FONT_SIZE)
    )]
    pub num_file: usize,

    /// Sum of individual file sizes declared in paths
    #[serde(rename = "Sum of file sizes (bytes)")]
    #[xlsx(
        value_format = Format::new()
            .set_align(FormatAlign::VerticalCenter)
            .set_num_format("#,##0")
            .set_font_name(FONT_NAME)
            .set_font_size(FONT_SIZE)
    )]
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
