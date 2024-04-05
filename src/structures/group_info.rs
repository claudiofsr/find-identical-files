use crate::{
    add_thousands_separator,
    args::{Arguments, ResultFormat::*},
    my_print, split_and_insert, FileExtension, FileInfo, Key, MyResult, PathBufExtension, PathInfo,
    TotalInfo, CSV_FILENAME, SEPARATOR,
};
use rayon::prelude::*;
use serde::Serialize;
use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::PathBuf,
    //thread,
};

/// Grouped file information
#[derive(Debug, Clone, Serialize)]
pub struct GroupInfo {
    /// Key Information
    #[serde(rename = "File information")]
    pub key: Key,
    /// File Paths
    #[serde(rename = "Paths")]
    pub paths: Vec<PathBuf>, // Arc<[PathBuf]> for immutable data
    /// Number of duplicate files with the same size and blake3 hash
    #[serde(rename = "Number of duplicate files")]
    pub num_file: usize,
    /// Sum of individual file sizes declared in paths
    #[serde(
        rename = "Sum of file sizes",
        serialize_with = "add_thousands_separator"
    )]
    pub sum_size: usize,
}

impl GroupInfo {
    /// Print GroupInfo fields in chosen format
    pub fn print_formatted(
        &self,
        arguments: &Arguments,
        write: &mut Box<&mut dyn Write>,
    ) -> MyResult<()> {
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
                writeln!(
                    write,
                    "size: {} bytes",
                    split_and_insert(self.key.size, SEPARATOR)
                )?;
                writeln!(write, "hash: {}", self.key.hash.clone().unwrap_or_default())?;
                writeln!(write, "Paths: {:#?}", self.paths)?;
                writeln!(write, "Number of duplicate files: {}", self.num_file)?;
                writeln!(
                    write,
                    "Sum of file sizes: {} bytes\n",
                    split_and_insert(self.sum_size, SEPARATOR)
                )?;
            }
        }

        Ok(())
    }

    /// Update hash
    pub fn update_hash(&self, opt_arguments: Option<&Arguments>) -> Vec<FileInfo> {
        self.paths
            .clone()
            .into_par_iter() // rayon parallel iterator
            .map(|path| {
                let key = match path.get_hash(opt_arguments) {
                    Ok(hash) => Key {
                        size: self.key.size,
                        hash,
                    },
                    Err(why) => {
                        eprintln!("fn update_hash()");
                        eprintln!("path: {:#?}", path.display());
                        panic!("Error getting path hash: {why}")
                    }
                };

                FileInfo { key, path }
            })
            .collect()
    }

    /// Update hash
    pub fn flatten(&self) -> Vec<PathInfo> {
        self.paths
            .iter()
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
    /**
    Get duplicate files from hashing first few bytes or whole file.

    If opt_arguments is None, get the hash of the first few bytes.

    If opt_arguments are Some, get whole file hash.
    */
    fn get_duplicate_files(&self, opt_arguments: Option<&Arguments>) -> Vec<GroupInfo>;

    /**
    Sort the list of duplicate files.

    Two options:

    1. Sort by number of duplicate files and then by (file size, hash);
    2. Sort by (file size, hash). `default`
    */
    fn sort_duplicate_files(&mut self, arguments: &Arguments);

    /// Print duplicate files
    fn print_duplicated_files(&self, arguments: &Arguments) -> MyResult<()>;

    /// Get Total Info
    fn get_total_info(&self, arguments: &Arguments, total_num_files: usize) -> TotalInfo;

    fn export_to_csv(&self, path: PathBuf) -> MyResult<()>;
}

impl GroupExtension for [GroupInfo] {
    fn get_duplicate_files(&self, opt_arguments: Option<&Arguments>) -> Vec<GroupInfo> {
        let duplicate_hash: Vec<GroupInfo> = self
            .par_iter() // rayon parallel iterator
            .flat_map(|group_info| group_info.update_hash(opt_arguments).get_grouped_files())
            .collect();

        duplicate_hash
    }

    fn sort_duplicate_files(&mut self, arguments: &Arguments) {
        if arguments.sort {
            // Sort by number of duplicate files and then by (file size, hash).
            self.par_sort_unstable_by_key(|group_info| {
                (
                    group_info.num_file,
                    group_info.key.size,
                    group_info.key.hash.clone(),
                )
            });
        } else {
            // Sort by (file size, hash) and then by number of duplicate files.
            self.par_sort_unstable_by_key(|group_info| {
                (group_info.key.size, group_info.key.hash.clone())
            });
        }
    }

    fn print_duplicated_files(&self, arguments: &Arguments) -> MyResult<()> {
        let all_buffer: Vec<u8> = self
            .par_chunks(rayon::current_num_threads())
            .flat_map(|groups_info| -> MyResult<Vec<u8>> {
                let mut buffer: Vec<u8> = Vec::new();
                let mut write: Box<&mut dyn Write> = Box::new(&mut buffer);
                groups_info
                    .iter()
                    .try_for_each(|group_info| -> MyResult<()> {
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
        let (total_num_duplicate, total_size) = rayon::join(
            || self.par_iter().map(|group_info| group_info.num_file).sum(),
            || self.par_iter().map(|group_info| group_info.sum_size).sum(),
        );

        /*
        let (result_a, result_b) = thread::scope(|s| {
            let thread_a = s.spawn(|| self.par_iter().map(|group_info| group_info.num_file).sum());
            let thread_b = s.spawn(|| self.par_iter().map(|group_info| group_info.sum_size).sum());

            // Wait for background thread to complete.
            // Call join() on each handle to make sure all the threads finish.
            // join() returns immediately when the associated thread completes.
            (thread_a.join(), thread_b.join())
        });

        let (total_num_duplicate, total_size) = match (result_a, result_b) {
            (Ok(sum_a), Ok(sum_b)) => (sum_a, sum_b),
            _ => panic!("thread::scope failed!"),
        };
        */

        TotalInfo {
            algorithm: arguments.algorithm,
            total_num_files,
            total_num_duplicate,
            total_num_hashes: self.len(),
            total_size,
        }
    }

    fn export_to_csv(&self, path: PathBuf) -> MyResult<()> {
        let path_csv: PathBuf = get_path_csv(path)?;

        // Open a file in write-only mode
        let file: File = match OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true) // replace the file
            .open(&path_csv)
        {
            Ok(f) => f,
            Err(error) => {
                eprintln!("fn export_to_csv()");
                eprintln!("Couldn't create {:?}", path_csv);
                panic!("Error: {error}");
            }
        };

        let mut writer = csv::WriterBuilder::new()
            .delimiter(b';')
            .has_headers(true)
            .quote_style(csv::QuoteStyle::Necessary) // NonNumeric
            .from_writer(file);

        for group_info in self {
            for path_info in group_info.flatten() {
                writer.serialize(path_info)?;
            }
        }

        writer.flush()?;

        Ok(())
    }
}

fn get_path_csv(path: PathBuf) -> MyResult<PathBuf> {
    let mut path_csv: PathBuf = if std::path::Path::new(&path).try_exists()? {
        path.to_path_buf()
    } else {
        eprintln!("fn export_to_csv()");
        panic!("The path {path:?} was not found!");
    };

    path_csv.push(CSV_FILENAME);

    eprintln!("Write CSV File: {:?}", path_csv);

    Ok(path_csv)
}
