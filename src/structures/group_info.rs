use crate::{
    CSV_FILENAME, FIFResult, FileExtension, FileInfo, Key, PathBufExtension, PathInfo, Procedure,
    TotalInfo, XLSX_FILENAME, add_thousands_separator,
    args::{Arguments, ResultFormat::*},
    get_thousands_separator, my_print, split_and_insert, write_xlsx,
};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::{fs::OpenOptions, io::Write, path::PathBuf};

/// Grouped file information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupInfo {
    /// File Paths
    #[serde(rename = "Paths")]
    pub paths: Vec<PathBuf>, // Vec<PathBuf> ; Arc<[PathBuf]> for immutable data

    /// Key Information (Size and Hash)
    #[serde(rename = "File information")]
    pub key: Key,

    /// Number of identical files in this group
    #[serde(rename = "Number of identical files")]
    pub num_file: usize,

    /// Sum of individual file sizes in this group
    #[serde(
        rename = "Sum of file sizes",
        serialize_with = "add_thousands_separator"
    )]
    pub sum_size: usize,
}

impl GroupInfo {
    /// Print GroupInfo fields in the chosen format (JSON, YAML, or Personal)
    pub fn print_formatted(
        &self,
        arguments: &Arguments,
        write: &mut Box<&mut dyn Write>,
    ) -> FIFResult<()> {
        let thousands_separator: char = get_thousands_separator();

        match &arguments.result_format {
            Json => {
                // Serialize GroupInfo to a JSON string.
                let serialized = serde_json::to_string_pretty(self)?;
                writeln!(write, "{serialized}\n")?;
            }
            Yaml => {
                // Serialize GroupInfo to a YAML string.
                let serialized = serde_yaml::to_string(self)?;
                writeln!(*write, "{serialized}")?;
            }
            Personal => {
                writeln!(write, "Paths: {:#?}", self.paths)?;
                writeln!(write, "Hash: {}", self.key.hash.clone().unwrap_or_default())?;
                writeln!(write, "Number of identical files: {}", self.num_file)?;
                writeln!(
                    write,
                    "Size of individual file: {} bytes",
                    split_and_insert(self.key.size, thousands_separator)
                )?;
                writeln!(
                    write,
                    "Sum of file sizes: {} bytes\n",
                    split_and_insert(self.sum_size, thousands_separator)
                )?;
            }
        }

        Ok(())
    }

    /// Updates the hash for all files in the group based on the current Procedure.
    ///
    /// This runs in parallel using Rayon. If any file fails to be hashed (e.g.,
    /// due to a sudden I/O error), the function returns a `FIFError` instead of panicking.
    pub fn update_hash(
        &self,
        arguments: &Arguments,
        procedure: Procedure,
    ) -> FIFResult<Vec<FileInfo>> {
        self.paths
            .par_iter() // Parallel iterator over PathBuf references
            .map(|path| {
                // get_hash already returns FIFResult<Option<String>>
                let hash = path.get_hash(arguments, procedure)?;

                Ok(FileInfo {
                    key: Key {
                        size: self.key.size,
                        hash,
                    },
                    path: path.clone(),
                })
            })
            .collect() // Magic of Rayon/Std: Collects Vec<Result> into Result<Vec>
    }

    /// Convert [`GroupInfo`] to a flat vector of [`PathInfo`]
    pub fn flatten(&self) -> Vec<PathInfo> {
        self.paths
            .par_iter()
            .map(|path| PathInfo {
                size: self.key.size,
                hash: self.key.hash.clone(),
                path: path.to_owned(),
                num_file: self.num_file,
                sum_size: self.sum_size,
            })
            .collect()
    }
}

pub trait GroupExtension {
    /// Filter and group files based on the hash (partial or entire) defined by the Procedure.
    fn get_identical_files(
        &self,
        arguments: &Arguments,
        procedure: Procedure,
    ) -> FIFResult<Vec<GroupInfo>>;

    /// Sort the list of identical files based on user arguments.
    fn sort_identical_files(&mut self, arguments: &Arguments);

    /// Print identical files to the standard output/buffer.
    fn print_identical_files(&self, arguments: &Arguments) -> FIFResult<()>;

    /// Calculate total statistics (count, size, etc.)
    fn get_total_info(&self, arguments: &Arguments, total_num_files: usize) -> TotalInfo;

    /// Convert Vec<[`GroupInfo`]> to Vec<[`PathInfo`]> for exporting
    fn get_path_info(&self) -> Vec<PathInfo>;

    /// Export identical file information to CSV format
    fn export_to_csv(&self, dir_path: PathBuf) -> FIFResult<()>;

    /// Export identical file information to XLSX format
    fn export_to_xlsx(&self, dir_path: PathBuf) -> FIFResult<()>;
}

impl GroupExtension for [GroupInfo] {
    /// Progressively filters groups by updating hashes and re-grouping.
    ///
    /// This implementation uses a Map-Reduce pattern with `try_fold` to minimize
    /// intermediate allocations and support early exit on I/O errors.
    fn get_identical_files(
        &self,
        arguments: &Arguments,
        procedure: Procedure,
    ) -> FIFResult<Vec<GroupInfo>> {
        self.par_iter()
            .try_fold(
                // 1. Creation: Each thread worker initializes its own local vector
                Vec::new,
                // 2. Folding: Process each group and accumulate results locally
                |mut local_accumulator, group_info| {
                    // Update hashes for the current group (short-circuits on Err)
                    let updated_files = group_info.update_hash(arguments, procedure)?;

                    // Group files based on the new hashes
                    let new_subgroups = updated_files.get_grouped_files(arguments, procedure);

                    // Append subgroups to the local thread vector
                    local_accumulator.extend(new_subgroups);

                    Ok(local_accumulator)
                },
            )
            .try_reduce(
                // 3. Identity: The base for merging results
                Vec::new,
                // 4. Reduction: Merge the vectors from different threads
                |mut vec_a, vec_b| {
                    vec_a.extend(vec_b);
                    Ok(vec_a)
                },
            )
    }

    fn sort_identical_files(&mut self, arguments: &Arguments) {
        if arguments.sort {
            // Sort by number of identical files and then by (file size, hash).
            self.par_sort_unstable_by_key(|group_info| {
                (
                    group_info.num_file,
                    group_info.key.size,
                    group_info.key.hash.clone(),
                )
            });
        } else {
            // Sort by (file size, hash) and then by number of identical files.
            self.par_sort_unstable_by_key(|group_info| {
                (group_info.key.size, group_info.key.hash.clone())
            });
        }
    }

    fn print_identical_files(&self, arguments: &Arguments) -> FIFResult<()> {
        let all_buffer: Vec<u8> = self
            .par_chunks(rayon::current_num_threads())
            .flat_map(|groups_info| -> FIFResult<Vec<u8>> {
                let mut buffer: Vec<u8> = Vec::new();
                let mut write: Box<&mut dyn Write> = Box::new(&mut buffer);
                groups_info
                    .iter()
                    .try_for_each(|group_info| -> FIFResult<()> {
                        group_info.print_formatted(arguments, &mut write)
                    })?;
                Ok(buffer)
            })
            .flatten()
            .collect();

        my_print(&all_buffer)?;
        Ok(())
    }

    fn get_total_info(&self, arguments: &Arguments, total_num_files: usize) -> TotalInfo {
        // Takes two closures and potentially runs them in parallel.
        let (total_num_identical, total_size) = rayon::join(
            || self.par_iter().map(|group_info| group_info.num_file).sum(),
            || self.par_iter().map(|group_info| group_info.sum_size).sum(),
        );

        /*
        let (result_a, result_b) = thread::scope(|s| {
            let thread_a = s.spawn(|| self.par_iter().map(|group_info| group_info.num_file).sum());
            let thread_b = s.spawn(|| self.par_iter().map(|group_info| group_info.sum_size).sum());

            // Wait for background thread to procedure.
            // Call join() on each handle to make sure all the threads finish.
            // join() returns immediately when the associated thread procedures.
            (thread_a.join(), thread_b.join())
        });

        let (total_num_identical, total_size) = match (result_a, result_b) {
            (Ok(sum_a), Ok(sum_b)) => (sum_a, sum_b),
            _ => panic!("thread::scope failed!"),
        };
        */

        TotalInfo {
            algorithm: arguments.algorithm,
            total_num_files,
            total_num_identical,
            total_num_hashes: self.len(),
            total_size,
        }
    }

    fn get_path_info(&self) -> Vec<PathInfo> {
        self.par_iter() // rayon parallel iterator
            .flat_map(|group_info| group_info.flatten())
            .collect()
    }

    fn export_to_csv(&self, mut dir_path: PathBuf) -> FIFResult<()> {
        dir_path.push(CSV_FILENAME); // dir_path + filename
        eprintln!("Write CSV File: {dir_path:?}");

        // Open a file in write-only mode
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&dir_path)
            .map_err(|e| {
                eprintln!("Failed to create CSV file at {dir_path:?}: {e}");
                e
            })?;

        let mut writer = csv::WriterBuilder::new()
            .delimiter(b';')
            .has_headers(true)
            .quote_style(csv::QuoteStyle::Necessary) // NonNumeric
            .from_writer(file);

        // Serialize path info into CSV
        for path_info in self.get_path_info() {
            writer.serialize(path_info)?;
        }

        writer.flush()?;

        Ok(())
    }

    fn export_to_xlsx(&self, mut dir_path: PathBuf) -> FIFResult<()> {
        dir_path.push(XLSX_FILENAME); // dir_path + filename
        eprintln!("Write XLSX File: {dir_path:?}");

        write_xlsx(&self.get_path_info(), "Identical Files", dir_path)?;

        Ok(())
    }
}
