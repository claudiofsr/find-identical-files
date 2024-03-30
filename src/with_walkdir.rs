use crate::{
    Arguments,
    MyResult,
    FileInfo,
    Key,
    get_path,
};

use std::{
    ops::RangeInclusive,
    path::PathBuf,
    process,
};

use walkdir::{
    DirEntry,
    WalkDir,
};

use rayon::prelude::*;

/// Get all files into one vector.
///
/// Use walkdir.
pub fn get_all_files(arguments: &Arguments) -> MyResult<Vec<FileInfo>> {

    // Set a minimum file size (in bytes) to search for duplicate files.
    let min_size: u64 = arguments.min_size.unwrap_or(0);

    // Set a maximum file size (in bytes) to search for duplicate files.
    let max_size: u64 = arguments.max_size.unwrap_or(std::u64::MAX);

    let size_range: RangeInclusive<u64> = min_size ..= max_size;

    let entries: Vec<DirEntry> = get_entries(arguments)?;

    let all_files: Vec<FileInfo> = entries
        .into_par_iter() // rayon parallel iterator
        //.iter()
        .filter_map(|entry| {
            if let Ok(metadata) = entry.metadata() {
                let size_u64: u64 = metadata.len();
                //let inode_number: u64 = metadata.ino();

                if size_range.contains(&size_u64) {
                    let key = Key::new(size_u64, None);
                    let path = entry.into_path();
                    Some(FileInfo {key, path})
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    Ok(all_files)
}

/// Get result: Vec<DirEntry>.
fn get_entries(arguments: &Arguments) -> MyResult<Vec<DirEntry>> {
    let path: PathBuf = get_path(arguments)?;

    // Set the minimum depth to search for duplicate files.
    let min_depth: usize = arguments.min_depth.unwrap_or(0);

    // Set the maximum depth to search for duplicate files.
    let max_depth: usize = arguments.max_depth.unwrap_or(std::usize::MAX);

    let entries: Vec<DirEntry> = WalkDir::new(path)
        .min_depth(min_depth)
        .max_depth(max_depth)
        .into_iter()
        .filter_entry(|e| !arguments.omit_hidden || !is_hidden(e))
        .map_while(|result| {
            match result {
                Ok(dir_entry) => Some(dir_entry),
                Err(why) => {
                    eprintln!("Error: {why}");
                    process::exit(1)
                }
            }
        })
        .filter(|entry| entry.file_type().is_file())
        .collect();

    Ok(entries)
}

// https://github.com/BurntSushi/walkdir
// https://rust-lang-nursery.github.io/rust-cookbook/file/dir.html
/// Identify hidden files efficiently on unix.
fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| entry.depth() != 0 && s.starts_with('.'))
        .unwrap_or(false)
}
