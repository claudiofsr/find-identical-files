use crate::{
    Algorithm, FIFResult, GroupInfo, add_thousands_separator,
    args::{Arguments, ResultFormat::*},
    get_thousands_separator, split_and_insert,
    structures::group_info::GroupExtension,
};
use serde::Serialize;

/// Summarize information for all files found in the directory
#[derive(Debug, Default, Clone, Serialize)]
pub struct TotalInfo {
    /// Hashing algorithm
    #[serde(rename = "Hashing algorithm")]
    pub algorithm: Algorithm,
    /// Total number of files found in the directory
    #[serde(rename = "Total number of files")]
    pub total_num_files: usize,
    /// Total number of identical files
    #[serde(rename = "Total number of identical files")]
    pub total_num_identical: usize,
    /// Total number of different blake3 hashes
    #[serde(rename = "Total number of different hashes")]
    pub total_num_hashes: usize,
    /// Total size of identical files
    #[serde(
        rename = "Total size of identical files",
        serialize_with = "add_thousands_separator"
    )]
    pub total_size: usize,
}

impl TotalInfo {
    /// Get the summary information.
    pub fn get_summary(
        identical_hash: &[GroupInfo],
        arguments: &Arguments,
        total_num_files: usize,
    ) -> Self {
        let (_result_display, total_info) = rayon::join(
            || -> FIFResult<()> {
                identical_hash.print_identical_files(arguments)?;
                Ok(())
            },
            || -> TotalInfo { identical_hash.get_total_info(arguments, total_num_files) },
        );

        total_info
    }

    /// Print the identicald files information.
    pub fn print_summary(&self, arguments: &Arguments) -> FIFResult<()> {
        let thousands_separator: char = get_thousands_separator();

        match &arguments.result_format {
            Json => {
                // Serialize TotalInfo to a JSON string.
                let serialized = serde_json::to_string_pretty(&self)?;
                println!("{serialized}\n");
            }
            Yaml => {
                // Serialize GroupInfo to a YAML string.
                let serialized = serde_yaml::to_string(&self)?;
                println!("{serialized}");
            }
            Personal => {
                println!("Hashing algorithm: {}", arguments.algorithm); // or self.algorithm
                println!("Total number of files: {}", self.total_num_files);
                println!(
                    "Total number of identical files: {}",
                    self.total_num_identical
                );
                println!(
                    "Total number of different hashes: {}",
                    self.total_num_hashes
                );
                println!(
                    "Total size of identical files: {} bytes\n",
                    split_and_insert(self.total_size, thousands_separator)?
                );
            }
        }
        Ok(())
    }
}
