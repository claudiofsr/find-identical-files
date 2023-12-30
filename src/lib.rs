mod algorithms;
mod args;
mod structures;

pub use self::{
    algorithms::*,
    args::*,
    structures::*,
};

use std::{
    fs,
    str,
    path::PathBuf,
    process::{self, Command},
    //cmp::Ordering,
    //os::unix::prelude::MetadataExt,
};

pub type MyError = Box<dyn std::error::Error + Send + Sync>;
pub type MyResult<T> = Result<T, MyError>;

#[cfg(feature = "walkdir")]
use walkdir::{DirEntry, WalkDir};

#[cfg(feature = "walkdir")]
use rayon::prelude::*;

#[cfg(not(feature = "walkdir"))]
use jwalk::{DirEntry, Parallelism, WalkDirGeneric};

/// Get all files into one vector. Use jwalk.
#[cfg(not(feature = "walkdir"))]
pub fn get_all_files(arguments: &Arguments) -> MyResult<Vec<FileInfo>> {

    let path: PathBuf = get_path(arguments)?;

    let max_depth: usize = match arguments.max_depth {
        Some(depth) => depth,
        None => std::usize::MAX,
    };

    let jwalk = WalkDirGeneric::<((), Option<FileInfo>)>::new(path)
        .max_depth(max_depth)
        .parallelism(Parallelism::RayonNewPool(rayon::current_num_threads()))
        .skip_hidden(arguments.omit_hidden)
        .process_read_dir(|_depth, _path, _read_dir_state, dir_entry_results| {
            analyze_dir_entry_results(dir_entry_results);
        });

    let all_files: MyResult<Vec<FileInfo>> = jwalk
        .into_iter()
        .map_while(|result| {
            match result {
                Ok(dir_entry) => Some(dir_entry),
                Err(why) => {
                    eprintln!("Error: {why}");
                    process::exit(1)
                }
            }
        })
        .filter_map(|dir_entry| dir_entry.client_state.map(Ok))
        .collect();

    all_files
}

/// Get all files into one vector. Use walkdir.
#[cfg(feature = "walkdir")]
pub fn get_all_files(arguments: &Arguments) -> MyResult<Vec<FileInfo>> {
    let path: PathBuf = get_path(arguments)?;

    let max_depth: usize = match arguments.max_depth {
        Some(depth) => depth,
        None => std::usize::MAX,
    };

    let entries: Vec<DirEntry> = WalkDir::new(path)
        .max_depth(max_depth)
        .into_iter()
        .filter_entry(|e| !arguments.omit_hidden || !is_hidden(e))
        // Silently skip directories that the owner of the
        // running process does not have permission to access.
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

#[cfg(not(feature = "walkdir"))]
type JwalkResults = Vec<Result<DirEntry<((), Option<FileInfo>)>, jwalk::Error>>;

// https://docs.rs/jwalk
// https://github.com/Byron/jwalk/blob/main/examples/du.rs
#[cfg(not(feature = "walkdir"))]
fn analyze_dir_entry_results(dir_entry_results: &mut JwalkResults) {

    // inode: “index nodes”
    // https://doc.rust-lang.org/std/os/unix/fs/trait.MetadataExt.html#tymethod.ino

    /*
    // 1. Custom sort
    dir_entry_results
    .sort_by(|a, b| match (a, b) {
        (Ok(a), Ok(b)) => a.metadata().map(|m| m.ino()).unwrap_or(0).cmp(&b.metadata().map(|m| m.ino()).unwrap_or(0)),
        (Ok(_), Err(_)) => Ordering::Less,
        (Err(_), Ok(_)) => Ordering::Greater,
        (Err(_), Err(_)) => Ordering::Equal,
    });
    */

    // 3. Custom skip
    dir_entry_results
        .iter_mut()
        //.par_iter_mut() // rayon parallel iterator
        .map_while(|result| {
            match result {
                Ok(dir_entry) => Some(dir_entry),
                Err(why) => {
                    eprintln!("Error: {why}");
                    process::exit(1)
                }
            }
        })
        .filter(|dir_entry| dir_entry.file_type().is_file())
        .for_each(|dir_entry| {
            if let Ok(size_u64) = dir_entry.metadata().map(|m| m.len()) {
                dir_entry.client_state = Some(FileInfo {
                    key: Key {
                        size: usize::try_from(size_u64).expect("try u64 -> usize failed!"),
                        hash: None,
                    },
                    path: dir_entry.path(),
                });
            }
        });
}

// https://github.com/BurntSushi/walkdir
// https://rust-lang-nursery.github.io/rust-cookbook/file/dir.html
/// Identify hidden files efficiently on unix.
#[cfg(feature = "walkdir")]
pub fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| entry.depth() != 0 && s.starts_with('.'))
        .unwrap_or(false)
}

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
