use crate::{
    add_thousands_separator,
    args::{Algorithm, Arguments, ResultFormat::*},
    split_and_insert,
    structures::group_info::GroupExtension,
    GroupInfo, MyResult,
};
use serde::Serialize;
use std::thread;

/// Summarize information for all files found in the directory
#[derive(Debug, Default, Clone, Serialize)]
pub struct TotalInfo {
    /// Hashing algorithm
    #[serde(rename = "Hashing algorithm")]
    pub algorithm: Algorithm,
    /// Total number of files found in the directory
    #[serde(rename = "Total number of files")]
    pub total_num_files: usize,
    /// Total number of duplicate files
    #[serde(rename = "Total number of duplicate files")]
    pub total_num_duplicate: usize,
    /// Total number of different blake3 hashes
    #[serde(rename = "Total number of different hashes")]
    pub total_num_hashes: usize,
    /// Total size of duplicate files
    #[serde(
        rename = "Total size of duplicate files",
        serialize_with = "add_thousands_separator"
    )]
    pub total_size: usize,
}

impl TotalInfo {
    /// Print the duplicated files and get the summary information.
    pub fn get_summary(
        duplicate_hash: &[GroupInfo],
        arguments: &Arguments,
        total_num_files: usize,
    ) -> Self {
        let (result_display, result_total_info) = thread::scope(|s| {
            let thread_a =
                s.spawn(|| -> MyResult<()> { duplicate_hash.print_duplicated_files(arguments) });
            let thread_b = s.spawn(|| -> TotalInfo {
                duplicate_hash.get_total_info(arguments, total_num_files)
            });

            // Wait for background thread to complete.
            // Call join() on each handle to make sure all the threads finish.
            // join() returns immediately when the associated thread completes.
            (thread_a.join(), thread_b.join())
        });

        let (_display, total_info) = match (result_display, result_total_info) {
            (Ok(display), Ok(total_info)) => (display, total_info),
            _ => panic!("thread::scope failed!"),
        };

        total_info
    }

    pub fn print_sumary(&self, arguments: &Arguments) -> MyResult<()> {
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
                    "Total number of duplicate files: {}",
                    self.total_num_duplicate
                );
                println!(
                    "Total number of different hashes: {}",
                    self.total_num_hashes
                );
                println!(
                    "Total size of duplicate files: {} bytes\n",
                    split_and_insert(self.total_size, '.')
                );
            }
        }
        Ok(())
    }
}
