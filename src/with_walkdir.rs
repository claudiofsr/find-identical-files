use crate::{Arguments, FIFError, FIFResult, FileInfo, Key, get_path};
use rayon::prelude::*;
use std::path::PathBuf;
use walkdir::{DirEntry, WalkDir};

/// Collects all files into a single Vector using the `walkdir` crate.
///
/// This function first collects all valid file entries sequentially and then
/// processes them in parallel using Rayon to determine which ones match
/// the size constraints and to initialize the `FileInfo` structures.
pub fn get_all_files(arguments: &Arguments) -> FIFResult<Vec<FileInfo>> {
    let entries: Vec<DirEntry> = get_entries(arguments)?;

    // Process collected entries in parallel.
    // We map to FIFResult<Option<FileInfo>> to capture potential conversion errors.
    let all_files: Vec<FileInfo> = entries
        .into_par_iter()
        .map(|entry| -> FIFResult<Option<FileInfo>> {
            // metadata() might fail if the file was deleted or permissions changed
            let metadata = entry.metadata().map_err(|e| FIFError::Io(e.into()))?;
            let file_size: u64 = metadata.len();

            if arguments.size_is_included(file_size) {
                // Key::new returns a FIFResult. If it fails, we propagate the error.
                let key = Key::new(file_size, None)?;
                let path = entry.into_path();

                Ok(Some(FileInfo { key, path }))
            } else {
                Ok(None)
            }
        })
        // If any thread returns an Err, collect will propagate the first error found.
        .collect::<FIFResult<Vec<Option<FileInfo>>>>()?
        .into_iter()
        .flatten() // Remove the None values
        .collect();

    Ok(all_files)
}

/// Traverses the directory and collects file entries into a Vector.
///
/// Filters are applied for hidden files and file types (keeping only regular files).
fn get_entries(arguments: &Arguments) -> FIFResult<Vec<DirEntry>> {
    let dir_path: PathBuf = get_path(arguments)?;

    let entries: Vec<DirEntry> = WalkDir::new(dir_path)
        .min_depth(arguments.min_depth)
        .max_depth(arguments.max_depth)
        .into_iter()
        // filter_entry stops recursion into hidden directories
        .filter_entry(|e| !arguments.omit_hidden || !is_hidden(e))
        .filter_map(|result| result.ok()) // Ignore walking errors (e.g., permission denied)
        .filter(|entry| entry.file_type().is_file())
        .collect();

    Ok(entries)
}

/// Efficiently identifies hidden files or directories on Unix-like systems.
///
/// A hidden entry is defined as one that starts with a dot ('.') and is not
/// the root directory itself.
fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| entry.depth() != 0 && s.starts_with('.'))
        .unwrap_or(false)
}
