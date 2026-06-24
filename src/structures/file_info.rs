use crate::{Arguments, GroupInfo, Key, Procedure};
use hashbrown::HashMap;
use rayon::prelude::*;
use std::path::PathBuf;

/// Individual file information
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// Key Information
    pub key: Key,

    /// File Path
    pub path: PathBuf,
}

pub trait FileExtension {
    /// Get two or more files with same key: (size, `Option<hash>`)
    fn get_grouped_files(&self, arguments: &Arguments, procedure: Procedure) -> Vec<GroupInfo>;
}

impl FileExtension for [FileInfo] {
    fn get_grouped_files(&self, arguments: &Arguments, procedure: Procedure) -> Vec<GroupInfo> {
        // minimum and maximum frequency (number of identical files)
        let min_frequency: usize = arguments.min_frequency as usize;
        let max_frequency: usize = arguments.max_frequency as usize;

        let mut group_by: HashMap<Key, Vec<PathBuf>> = HashMap::new();

        self.iter().for_each(|file_info| {
            group_by
                // key: (size, Option<hash>), value: paths
                .entry(file_info.key.clone())
                // If there's no entry for the key, create a new Vec and return a mutable ref to it
                .or_default()
                // and insert the item onto the Vec
                .push(file_info.path.clone())
        });

        // Converting group_by to vector
        let grouped_files: Vec<GroupInfo> = group_by
            .into_par_iter() // rayon parallel iterator
            .filter(|(_key, paths)| {
                procedure.is_valid_frequency(paths.len(), min_frequency, max_frequency)
            })
            .map(|(key, paths)| {
                let num_file = paths.len();
                let sum_size = key.size * num_file;
                GroupInfo {
                    key,
                    paths,
                    num_file,
                    sum_size,
                }
            })
            .collect();

        grouped_files
    }
}
