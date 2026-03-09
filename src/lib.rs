mod args;
mod enumerations;
mod error;
mod excel;
mod separator;
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
    enumerations::procedures::*,
    error::*,
    separator::get_thousands_separator,
    structures::file_info::{FileExtension, FileInfo},
    structures::group_info::{GroupExtension, GroupInfo},
    structures::key_info::Key,
    structures::path_info::PathInfo,
    structures::total_info::TotalInfo,
};
pub use excel::write_xlsx;
use serde::Serializer;
use std::{
    fmt::{self, Write as FmtWrite}, // Rename to avoid conflict
    fs::{self, File},
    io::{self, Write as IoWrite}, // Rename to avoid conflict
    path::{Path, PathBuf},
    process::Command,
    str,
};

pub const CSV_FILENAME: &str = "fif.csv";
pub const XLSX_FILENAME: &str = "fif.xlsx";

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

/// Prints the provided byte buffer to standard output as a UTF-8 string.
///
/// # Errors
/// Returns `FIFError::Utf8Error` if the buffer contains invalid UTF-8 sequences.
/// Returns `FIFError::Io` if writing to stdout fails.
pub fn my_print(buffer: &[u8]) -> FIFResult<()> {
    // Attempt to convert the raw byte slice into a valid UTF-8 string slice.
    // The '?' operator will catch any Utf8Error and wrap it into FIFError.
    let print_msg = str::from_utf8(buffer)?;

    // Print to standard output.
    print!("{print_msg}");

    // Optional but recommended: flush stdout to ensure the output is displayed immediately.
    io::stdout().flush()?;

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
///
/// This internal function is generic over any 'Writer' (String, Formatter, etc.).
/// It avoids code duplication while maintaining maximum performance.
fn write_integer_with_separator<W: FmtWrite>(
    integer: usize,
    separator: char,
    writer: &mut W,
) -> fmt::Result {
    let s = integer.to_string();
    let bytes = s.as_bytes();
    let len = bytes.len();

    for (i, &byte) in bytes.iter().enumerate() {
        // Apply thousands separator every 3 digits from the right.
        if i > 0 && (len - i).is_multiple_of(3) {
            writer.write_char(separator)?;
        }
        writer.write_char(byte as char)?;
    }
    Ok(())
}

/// High-performance string formatter returning a custom Result.
///
/// Performance wins:
/// 1. Exact capacity allocation (Single Heap Allocation).
/// 2. Automatic error propagation via `?` (Requires `From<fmt::Error>` in FIFError).
pub fn split_and_insert(integer: usize, separator: char) -> FIFResult<String> {
    let s_val = integer.to_string();
    let len = s_val.len();

    // Small optimization: skip processing if no separators are needed.
    if len <= 3 {
        return Ok(s_val);
    }

    // Step 1: Exact Capacity Calculation.
    // Length of digits + (Number of separators * bytes per separator).
    let num_seps = (len - 1) / 3;
    let final_capacity = len + (num_seps * separator.len_utf8());

    // Step 2: Allocate memory once.
    let mut result = String::with_capacity(final_capacity);

    // Step 3: Write digits directly into the allocated buffer.
    // The '?' operator works here because FIFError implements From<std::fmt::Error>.
    write_integer_with_separator(integer, separator, &mut result)?;

    Ok(result)
}

/// Internal helper to format values directly into a stream.
/// This avoids allocating a `String` inside the Serializer.
struct BytesFormatter(usize);

impl fmt::Display for BytesFormatter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sep = get_thousands_separator();

        // Write digits directly to the output stream (f).
        write_integer_with_separator(self.0, sep, f)?;

        // Append suffix directly to the output stream.
        f.write_str(" bytes")
    }
}

/// Serde Serializer: Formats usize as a string with separators (e.g., "1.234 bytes").
/// Highly efficient: Writes directly to the serializer's buffer without
/// temporary String allocations on the heap.
pub fn add_thousands_separator<S>(size: &usize, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Use collect_str to pipe our Display implementation into the Serializer.
    serializer.collect_str(&BytesFormatter(*size))
}

#[cfg(test)]
mod tests_lib {
    use super::*;

    #[test]
    /// cargo test -- --show-output split_integer_into_groups
    fn split_integer_into_groups() -> FIFResult<()> {
        let mut result: Vec<String> = Vec::new();

        for integer in [
            0, 1, 12, 999, 1000, 1001, 1234, 12345, 123456, 1234567, 12345678,
        ] {
            let integer_splitted: String = split_and_insert(integer, '_')?;
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
        Ok(())
    }
}
