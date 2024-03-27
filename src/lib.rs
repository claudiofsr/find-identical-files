mod algorithms;
mod args;
mod structures;

#[cfg(feature = "walkdir")]
mod with_walkdir;

// default: use jwalk
#[cfg(not(feature = "walkdir"))]
mod with_jwalk;

#[cfg(feature = "walkdir")]
pub use with_walkdir::get_all_files;

#[cfg(not(feature = "walkdir"))]
pub use with_jwalk::get_all_files;

pub use self::{
    algorithms::{set_env_variables, PathBufExtension},
    args::Arguments,
    structures::key_info::Key,
    structures::file_info::FileInfo,
    structures::group_info::{GroupInfo, GroupExtension},
    structures::total_info::TotalInfo,
};

use hashbrown::HashMap;
use rayon::prelude::*;
use serde::Serializer;
use std::{
    fs,
    str,
    path::PathBuf,
    process::Command,
};

pub type MyError = Box<dyn std::error::Error + Send + Sync>;
pub type MyResult<T> = Result<T, MyError>;

/// Get path from arguments or from default (current directory).
pub fn get_path(arguments: &Arguments) -> MyResult<PathBuf> {
    let path: PathBuf = match &arguments.path {
        Some(path) => {
            if std::path::Path::new(path).try_exists()? {
                path.to_path_buf()
            } else {
                eprintln!("The path {path:?} was not found!");
                panic!("fn get_path()");
            }
        }
        None => PathBuf::from("."),
    };

    if arguments.full_path {
        Ok(fs::canonicalize(path)?) // full path
    } else {
        Ok(path) // relative path
    }
}

/// Print buffer to stdout
pub fn my_print(buffer: &[u8]) -> MyResult<()> {
    // Converts a slice of bytes to a string slice
    let print_msg = match str::from_utf8(buffer) {
        Ok(valid_uft8) => valid_uft8,
        Err(error) => {
            eprintln!("fn my_print()");
            eprintln!("Invalid UTF-8 sequence!");
            panic!("{error}");
        }
    };

    // Print to stdout
    print!("{print_msg}");
    Ok(())
}

/// Get two or more files with same key: (size, Option<hash>)
pub fn get_grouped_files(files_info: &[FileInfo]) -> Vec<GroupInfo> {
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
/// If opt_arguments is None, get the hash of the first few bytes.
///
/// If opt_arguments are Some, get whole file hash.
pub fn get_duplicate_files(duplicate_size: &[GroupInfo], opt_arguments: Option<&Arguments>) -> Vec<GroupInfo> {
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
                            hash: path.get_hash(opt_arguments).expect("get_hash() failed!"),
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

// https://stackoverflow.com/questions/34837011/how-to-clear-the-terminal-screen-in-rust-after-a-new-line-is-printed
// https://stackoverflow.com/questions/65497187/cant-run-a-system-command-in-windows
// Remove unwanted characters
// clear | cat -v ; echo
// ^[[H^[[2J^[[3J
/// Clear the terminal screen
pub fn clear_terminal_screen() {
    if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/c", "cls"])
            .spawn()
            .expect("cls command failed to start")
            .wait()
            .expect("failed to wait");
    } else {
        Command::new("clear")
            .spawn()
            .expect("clear command failed to start")
            .wait()
            .expect("failed to wait");
    };
}

/// Split integer and insert thousands separator
pub fn split_and_insert(integer: usize, insert: char) -> String {
    let group_size = 3;
    let integer_str = integer.to_string();

    if integer <= 999 {
        return integer_str;
    }

    let string_splitted: String = integer_str
        .chars()
        .enumerate()
        .flat_map(|(i, c)| {
            if (integer_str.len() - i) % group_size == 0 && i > 0 {
                Some(insert)
            } else {
                None
            }
            .into_iter()
            .chain(std::iter::once(c))
        })
        .collect::<String>();

    string_splitted
}

/// Serialize usize with fn split_and_insert().
pub fn add_thousands_separator<S>(size: &usize, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.collect_str(&format!("{} bytes", &split_and_insert(*size, '.')))
}

#[cfg(test)]
mod test_lib {
    use super::*;

    #[test]
    fn split_integer_into_groups() {
        // cargo test -- --show-output split_integer_into_groups

        let mut result: Vec<String> = Vec::new();

        for integer in [
            0, 1, 12, 999, 1000, 1001, 1234, 12345, 123456, 1234567, 12345678,
        ] {
            let integer_splitted: String = split_and_insert(integer, '.');
            println!("integer: {integer:<8} ; with thousands sep: {integer_splitted}");
            result.push(integer_splitted);
        }

        let valid = vec![
            "0",
            "1",
            "12",
            "999",
            "1.000",
            "1.001",
            "1.234",
            "12.345",
            "123.456",
            "1.234.567",
            "12.345.678",
        ];

        assert_eq!(valid, result);
    }
}
