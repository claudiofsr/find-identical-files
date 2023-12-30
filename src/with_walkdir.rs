use crate::{
    Arguments,
    MyResult,
    FileInfo,
    Key,
    get_path,
};

use std::{
    fs,
    process,
    path::PathBuf,
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

    let entries: Vec<DirEntry> = get_entries(arguments)?;

    let all_files: MyResult<Vec<FileInfo>> = entries
        .into_par_iter() // rayon parallel iterator
        //.iter()
        .map(|entry| {
            let path: PathBuf = entry.into_path();
            let metadata = fs::metadata(path.clone())?;
            Ok(FileInfo {
                key: Key {
                    size: usize::try_from(metadata.len())?,
                    hash: None,
                },
                path,
            })
        })
        .collect();

    all_files
}

/// Get result: Vec<DirEntry>.
pub fn get_entries(arguments: &Arguments) -> MyResult<Vec<DirEntry>> {
    let path: PathBuf = get_path(arguments)?;

    let max_depth: usize = match arguments.max_depth {
        Some(depth) => depth,
        None => std::usize::MAX,
    };

    let entries: Vec<DirEntry> = WalkDir::new(path)
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
pub fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| entry.depth() != 0 && s.starts_with('.'))
        .unwrap_or(false)
}
