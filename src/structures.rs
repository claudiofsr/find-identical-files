use crate::{
    my_print,
    split_and_insert,
    MyResult,
    args::{
        Arguments,
        ResultFormat::*,
    },
};
use serde::{
    Serialize,
    Serializer,
};
use std::{
    thread,
    ops::Add,
    io::Write,
    iter::Sum,
    path::PathBuf,
};
use rayon::prelude::*;

/*
Structures defined in this file:
    Key,
    FileInfo,
    GroupInfo,
    TotalInfo,
*/

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

/// Individual file information
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// Key Information
    pub key: Key,
    /// File Path
    pub path: PathBuf,
}

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

/// Summarize information for all files found in the directory
#[derive(Debug, Default, Clone, Serialize)]
pub struct TotalInfo {
    /// Hashing algorithm
    #[serde(rename = "Hashing algorithm")]
    pub algorithm: String,
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
    #[serde(rename = "Total size of duplicate files", serialize_with = "add_thousands_separator")]
    pub total_size: usize,
}

fn add_thousands_separator<S>(size: &usize, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.collect_str(&format_args!("{} bytes", &split_and_insert(*size, '.')))
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

pub trait Extensions {
    fn sort_duplicate_files(&mut self, arguments: &Arguments);
    fn print_duplicated_files(&self, arguments: &Arguments) -> MyResult<()>;
    fn get_total_info(&self, arguments: &Arguments, total_num_files: usize) -> MyResult<TotalInfo>;
}

impl Extensions for [GroupInfo] {
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
    fn get_total_info(&self, arguments: &Arguments, total_num_files: usize) -> MyResult<TotalInfo> {
        let mut total_info: TotalInfo = self
            .into_par_iter() // rayon parallel iterator
            .map(TotalInfo::new)
            .sum();

        total_info.algorithm = arguments.algorithm.to_string();
        total_info.total_num_files = total_num_files;

        Ok(total_info)
    }
}

/// Implement sum of TotalInfo elements for rayon usage
impl Sum for TotalInfo {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        iter.fold(Self::default(), |acc, x| acc + x)
    }
}

impl Add for TotalInfo {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            algorithm:           self.algorithm,
            total_num_files:     self.total_num_files     + other.total_num_files,
            total_num_duplicate: self.total_num_duplicate + other.total_num_duplicate,
            total_num_hashes:    self.total_num_hashes    + other.total_num_hashes,
            total_size:          self.total_size          + other.total_size,
        }
    }
}

impl TotalInfo {
    pub fn new(group_info: &GroupInfo) -> Self {
        TotalInfo {
            algorithm: "".to_string(),
            total_num_files: 1,
            total_num_duplicate: group_info.num_file,
            total_num_hashes: 1,
            total_size: group_info.sum_size,
        }
    }

    /// Print the duplicated files and get the summary information.
    pub fn get_summary(duplicate_hash: &[GroupInfo], arguments: &Arguments, total_num_files: usize) -> Self {
        let (result_display, result_total_info) = thread::scope(|s| {
            let thread_a = s.spawn(|| duplicate_hash.print_duplicated_files(arguments).unwrap());
            let thread_b = s.spawn(|| duplicate_hash.get_total_info(arguments, total_num_files).unwrap());

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
                println!("Total number of duplicate files: {}", self.total_num_duplicate);
                println!("Total number of different hashes: {}", self.total_num_hashes);
                println!("Total size of duplicate files: {} bytes\n", split_and_insert(self.total_size, '.'));
            }
        }
        Ok(())
    }
}
