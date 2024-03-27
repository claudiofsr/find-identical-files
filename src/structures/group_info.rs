use serde::Serialize;
use crate::add_thousands_separator;

use crate::{
    my_print,
    split_and_insert,
    args::{
        Arguments,
        ResultFormat::*,
    },
    Key,
    MyResult,
    TotalInfo,
};

use std::{
    io::Write,
    path::PathBuf,
};

use rayon::prelude::*;

/// Grouped file information
#[derive(Debug, Clone, Serialize)]
pub struct GroupInfo {
    /// Key Information
    #[serde(rename = "File information")]
    pub key: Key,
    /// File Paths
    #[serde(rename = "Paths")]
    pub paths: Vec<PathBuf>,
    /// Number of duplicate files with the same size and blake3 hash
    #[serde(rename = "Number of duplicate files")]
    pub num_file: usize,
    /// Sum of individual file sizes declared in paths
    #[serde(rename = "Sum of file sizes", serialize_with = "add_thousands_separator")]
    pub sum_size: usize,
}

impl GroupInfo {
    /// Print GroupInfo fields in chosen format
    pub fn print_formatted(&self, arguments: &Arguments, write: &mut Box<&mut dyn Write>) {
        match &arguments.result_format {
            Json => {
                // Serialize GroupInfo to a JSON string.
                let serialized = serde_json::to_string_pretty(self).unwrap();
                writeln!(write, "{serialized}\n").unwrap();
            }
            Yaml => {
                // Serialize GroupInfo to a YAML string.
                let serialized = serde_yaml::to_string(self).unwrap();
                writeln!(*write, "{serialized}").unwrap();
            }
            Personal => {
                writeln!(write, "size: {} bytes", split_and_insert(self.key.size, '.')).unwrap();
                writeln!(write, "hash: {}", self.key.hash.clone().unwrap_or_default()).unwrap();
                writeln!(write, "Paths: {:#?}", self.paths).unwrap();
                writeln!(write, "Number of duplicate files: {}", self.num_file).unwrap();
                writeln!(write, "Sum of file sizes: {} bytes\n", split_and_insert(self.sum_size, '.')).unwrap();
            }
        }
    }
}

pub trait GroupExtension {
    fn sort_duplicate_files(&mut self, arguments: &Arguments);
    fn print_duplicated_files(&self, arguments: &Arguments) -> MyResult<()>;
    fn get_total_info(&self, arguments: &Arguments, total_num_files: usize) -> TotalInfo;
}

impl GroupExtension for [GroupInfo] {
    /// Sort the list of duplicate files.
    ///
    /// Two options:
    /// 1. Sort by (file size, hash) and then by number of duplicate files;
    /// 2. Sort by number of duplicate files and then by (file size, hash).
    fn sort_duplicate_files(&mut self, arguments: &Arguments) {
        if arguments.sort {
            // Sort by (file size, hash) and then by number of duplicate files.
            self.par_sort_unstable_by_key(|group_info| {
                (
                    group_info.key.size,
                    group_info.key.hash.clone(),
                    group_info.num_file,
                )
            });
        } else {
            // Sort by number of duplicate files and then by (file size, hash).
            self.par_sort_unstable_by_key(|group_info| {
                (
                    group_info.num_file,
                    group_info.key.size,
                    group_info.key.hash.clone(),
                )
            });
        }
    }

    /// Print duplicate files
    fn print_duplicated_files(&self, arguments: &Arguments) -> MyResult<()> {
        let all_buffer: Vec<u8> = self
            .par_chunks(rayon::current_num_threads())
            .flat_map(|groups_info| {
                let mut buffer: Vec<u8> = Vec::new();
                let mut write: Box<&mut dyn Write> = Box::new(&mut buffer);
                groups_info
                    .iter()
                    .for_each(|group_info| group_info.print_formatted(arguments, &mut write));
                buffer
            })
            .collect();

        my_print(&all_buffer)?;
        Ok(())
    }

    /// Get Total Info
    fn get_total_info(&self, arguments: &Arguments, total_num_files: usize) -> TotalInfo {
        // Takes two closures and potentially runs them in parallel.
        let (total_num_duplicate, total_size) = rayon::join(
            || self.par_iter().map(|group_info| group_info.num_file).sum(),
            || self.par_iter().map(|group_info| group_info.sum_size).sum(),
        );

        TotalInfo {
            algorithm: arguments.algorithm,
            total_num_files,
            total_num_duplicate,
            total_num_hashes: self.len(),
            total_size,
        }
    }
}
