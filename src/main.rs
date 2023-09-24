use find_duplicate_files::*;
use hashbrown::HashMap;
use rayon::prelude::*;
use std::{
    error::Error,
    path::PathBuf,
    time::Instant,
};

fn main() -> Result<(), Box<dyn Error>> {
    set_stack_size();
    let time = Instant::now();
    let arguments = Arguments::build();

    // Get useful (duplicate) and useless (non-duplicate) files.
    let all_files: Vec<FileInfo> = get_all_files(&arguments)?;

    if arguments.verbose {
        eprintln!("0. all_file size: {}, time_elapsed: {:?}", all_files.len(), time.elapsed());
    }

    // To skip useless files, 3 procedures will be performed:

    // 1. Group files by size such that the key: (size, None);
    // Ignore filegroups containing only one file.
    let duplicate_size: Vec<GroupInfo> = get_grouped_files(&all_files);

    if arguments.verbose {
        eprintln!("1. duplicate_size: {}, time_elapsed: {:?}", duplicate_size.len(), time.elapsed());
    }

    // 2. Group files by first bytes such that the key: (size, Some(bytes));
    // Ignore filegroups containing only one file.
    let duplicate_bytes: Vec<GroupInfo> = get_duplicate_files(&duplicate_size, None);

    if arguments.verbose {
        eprintln!("2. duplicate_bytes: {}, time_elapsed: {:?}", duplicate_bytes.len(), time.elapsed());
    }

    // 3. Group files by hash such that the key: (size, Some(hash)).
    // Ignore filegroups containing only one file.
    let mut duplicate_hash: Vec<GroupInfo> = get_duplicate_files(&duplicate_bytes, Some(&arguments));

    if arguments.verbose {
        eprintln!("3. duplicate_hash: {}, time_elapsed: {:?}", duplicate_hash.len(), time.elapsed());
    }

    // Sort the list of duplicate files.
    duplicate_hash.sort_duplicate_files(&arguments);

    // Print the duplicated files and the summary information.
    TotalInfo::get_summary(&duplicate_hash, &arguments, all_files.len())
        .print_sumary(&arguments)?;

    if arguments.time {
        println!("Total Execution Time: {:?}", time.elapsed());
    }

    Ok(())
}

/// Get two or more files with same key: (size, Option<hash>)
fn get_grouped_files(files_info: &[FileInfo]) -> Vec<GroupInfo> {
    let mut group_by: HashMap<Key, Vec<PathBuf>> = HashMap::new();

    files_info.iter().for_each(|file_info| {
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
        .filter(|(_key, paths)| paths.len() > 1) // filter duplicate files with same key
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

/// Get duplicate files from hashing first few bytes or whole file.
///
/// If arguments is None, get the hash of the first few bytes.
///
/// If arguments are Some, get whole file hash.
fn get_duplicate_files(duplicate_size: &[GroupInfo], arguments: Option<&Arguments>) -> Vec<GroupInfo> {
    let duplicate_hash: Vec<GroupInfo> = duplicate_size
        .par_iter() // rayon parallel iterator
        .filter_map(|group_info| {
            let hashed_files: Vec<FileInfo> = group_info
                .paths
                .clone()
                .into_par_iter() // rayon parallel iterator
                .map(|path| {
                    FileInfo {
                        key: Key {
                            size: group_info.key.size,
                            hash: path.get_hash(arguments).expect("get_hash() failed!"),
                        },
                        path,
                    }
                })
                .collect();

            let duplicate_hash: Vec<GroupInfo> = get_grouped_files(&hashed_files);

            if !duplicate_hash.is_empty() {
                Some(duplicate_hash)
            } else {
                None
            }
        })
        .flatten()
        .collect();

    duplicate_hash
}