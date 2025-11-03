mod args;
mod enumerations;
mod error;
mod excel;
mod structures;
mod traits;

// https://crates.io/crates/cfg-if
cfg_if::cfg_if! {
    if #[cfg(feature = "walkdir")] {
        mod with_walkdir;
        pub use with_walkdir::get_all_files;
    } else {
        // default: use jwalk
        mod with_jwalk;
        pub use with_jwalk::get_all_files;
    }
}

pub use self::{
    args::Arguments,
    enumerations::algo::{Algorithm, PathBufExtension, SliceExtension},
    error::*,
    structures::file_info::{FileExtension, FileInfo},
    structures::group_info::{GroupExtension, GroupInfo},
    structures::key_info::Key,
    structures::path_info::PathInfo,
    structures::total_info::TotalInfo,
};
pub use excel::write_xlsx;
use serde::Serializer;
use std::{
    fs::{self, File},
    io,
    path::{Path, PathBuf},
    process::Command,
    str,
};

const STACK_SIZE: usize = 64 * 1024 * 1024;
pub const CSV_FILENAME: &str = "fif.csv";
pub const XLSX_FILENAME: &str = "fif.xlsx";

#[cfg(not(any(
    feature = "thousands-sep-comma",
    feature = "thousands-sep-dot",
    feature = "thousands-sep-space"
)))]
const THOUSANDS_SEPARATOR: char = ','; // Default para vírgula se nada for especificado

#[cfg(feature = "thousands-sep-comma")]
const THOUSANDS_SEPARATOR: char = ',';

#[cfg(feature = "thousands-sep-dot")]
const THOUSANDS_SEPARATOR: char = '.';

#[cfg(feature = "thousands-sep-space")]
const THOUSANDS_SEPARATOR: char = ' ';

/**
If `thread '<unknown>' has overflowed its stack`, set the stack size to a new value.

The default rustc stack size for main thread is currently `8 * 1024 * 1024`.

Stack size can be changed during compile time.

<https://github.com/rust-lang/rust/blob/master/compiler/rustc_interface/src/util.rs#L132>

<https://github.com/rust-lang/rust/issues/111897>

<https://doc.rust-lang.org/stable/std/thread/index.html#stack-size>
*/
pub fn set_env_variables() {
    // TODO: Audit that the environment access only happens in single-threaded code.
    unsafe { std::env::set_var("RUST_MIN_STACK", STACK_SIZE.to_string()) };
}

/// Opens a file in read-only mode.
///
/// Provides more informative error messages in case of failure.
pub fn open_file<P>(path: &P) -> FIFResult<File>
where
    P: AsRef<Path> + std::fmt::Debug,
{
    fs::OpenOptions::new()
        .read(true)
        .write(false) // This option, when false, will indicate that the file should not be writable if opened.
        .create(false) // No files will be created
        .open(path.as_ref())
        .map_err(|error| {
            let path_buf = path.as_ref().to_path_buf();
            match error.kind() {
                io::ErrorKind::NotFound => FIFError::FileNotFound { path: path_buf },
                io::ErrorKind::PermissionDenied => FIFError::PermissionDenied { path: path_buf },
                _ => FIFError::FileOpenError {
                    path: path_buf,
                    io_error: error,
                },
            }
        })
}

/// Get path from arguments or from default (current directory).
pub fn get_path(arguments: &Arguments) -> FIFResult<PathBuf> {
    let path: PathBuf = match &arguments.input_dir {
        Some(path) => path.to_owned(),
        None => PathBuf::from("."),
    };

    if arguments.extended_path {
        Ok(fs::canonicalize(path)?) // full path
    } else {
        Ok(path) // relative path
    }
}

/// Print buffer to stdout
pub fn my_print(buffer: &[u8]) -> FIFResult<()> {
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

/// Clear (wipe) the terminal screen
pub fn clear_terminal_screen() {
    let result = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/c", "cls"]).spawn()
    } else {
        // "clear" or "tput reset"
        Command::new("tput").arg("reset").spawn()
    };

    // Alternative solution:
    if result.is_err() {
        print!("{esc}c", esc = 27 as char);
    }
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
            if (integer_str.len() - i).is_multiple_of(group_size) && i > 0 {
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
    serializer.collect_str(&format!(
        "{} bytes",
        &split_and_insert(*size, THOUSANDS_SEPARATOR)
    ))
}

#[cfg(test)]
mod tests_lib {
    use super::*;

    #[test]
    /// cargo test -- --show-output split_integer_into_groups
    fn split_integer_into_groups() {
        let mut result: Vec<String> = Vec::new();

        for integer in [
            0, 1, 12, 999, 1000, 1001, 1234, 12345, 123456, 1234567, 12345678,
        ] {
            let integer_splitted: String = split_and_insert(integer, '_');
            println!("integer: {integer:<8} ; with thousands sep: {integer_splitted}");
            result.push(integer_splitted);
        }

        let valid = vec![
            "0",
            "1",
            "12",
            "999",
            "1_000",
            "1_001",
            "1_234",
            "12_345",
            "123_456",
            "1_234_567",
            "12_345_678",
        ];

        assert_eq!(valid, result);
    }
}
