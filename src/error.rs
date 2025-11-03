use crate::traits::Colors;
use rust_xlsxwriter::XlsxError;
use std::{io, path::PathBuf};
use thiserror::Error;

/**
Result type to simplify function signatures.

This is a custom result type that uses our custom `FIFError` for the error type.

Functions can return `FIFResult<T>` and then use `?` to automatically propagate errors.
*/
pub type FIFResult<T> = Result<T, FIFError>;

/// FIF Error enum
///
/// The `FIFError` enum defines the error values
///
/// <https://doc.rust-lang.org/rust-by-example/error/multiple_error_types/define_error_type.html>
#[derive(Error, Debug)]
pub enum FIFError {
    // Errors encountered while parsing CSV data (e.g., inconsistent columns, invalid data).
    #[error("{msg}: '{0}'", msg = "CSV Parsing Error".red().bold())]
    CSVError(#[from] csv::Error),

    /// Specific error when a file is not found.
    #[error("{msg}: '{path:?}'\nPerhaps some temporary files no longer exist!", msg = "File Not Found Error".red().bold())]
    FileNotFound { path: PathBuf },

    /// Generic file opening error with more context.
    #[error("{msg}: '{path:?}'\n{io_error}", msg = "File Open Error".red().bold())]
    FileOpenError {
        path: PathBuf,
        #[source]
        io_error: io::Error,
    },

    /// Standard I/O error wrapper.
    #[error("{msg}: '{0}'", msg = "IO Error".red().bold())]
    Io(#[from] io::Error),

    /// Error when a JSON serialization or deserialization operation fails.
    #[error("{msg}: '{0}'", msg = "JSON Serialization/Deserialization Error".red().bold())]
    Json(#[from] serde_json::Error),

    /// Error when a Yaml serialization or deserialization operation fails.
    #[error("{msg}: '{0}'", msg = "YAML Serialization/Deserialization Error".red().bold())]
    Yaml(#[from] serde_yaml::Error),

    /// Specific error when file permission is denied.
    #[error("{msg}: '{path:?}'", msg = "Permission Denied Error".red().bold())]
    PermissionDenied { path: PathBuf },

    /// XlsxError wrapper.
    #[error("{msg}: '{0}'", msg = "XLSX Error".red().bold())]
    XlsxError(#[from] XlsxError),
}

#[cfg(test)]
mod error_tests {
    use super::*;
    use std::io::ErrorKind;

    #[test]
    fn test_file_not_found_error_message() {
        let path = PathBuf::from("/non/existent/foo/file.txt");
        let error = FIFError::FileNotFound { path: path.clone() };

        let msg = "File Not Found Error".red().bold();
        let expected_msg = format!(
            "{msg}: '{:?}'\nPerhaps some temporary files no longer exist!",
            path
        );
        assert_eq!(format!("{}", error), expected_msg);
    }

    #[test]
    fn test_file_open_error_message() {
        let path = PathBuf::from("/var/log/some_protected_file.log");
        let io_error = io::Error::new(ErrorKind::PermissionDenied, "permission denied");
        let error = FIFError::FileOpenError {
            path: path.clone(),
            io_error,
        };

        let msg = "File Open Error".red().bold();
        let expected_msg = format!("{msg}: '{:?}'\npermission denied", path);
        assert_eq!(format!("{}", error), expected_msg);
    }

    #[test]
    fn test_csv_error_message() {
        // Create a std::io::Error from the ErrorKind, then convert it to csv::Error
        let io_error = io::Error::new(ErrorKind::InvalidData, "invalid data");
        let csv_error = csv::Error::from(io_error);
        let error = FIFError::CSVError(csv_error);

        let msg = "CSV Parsing Error".red().bold();
        let expected_msg = format!("{msg}: 'invalid data'");
        assert_eq!(format!("{}", error), expected_msg);
    }

    #[test]
    fn test_permission_denied_error_message() {
        let path = PathBuf::from("/etc/passwd");
        let error = FIFError::PermissionDenied { path: path.clone() };

        let msg = "Permission Denied Error".red().bold();
        let expected_msg = format!("{msg}: '{path:?}'");
        assert_eq!(format!("{}", error), expected_msg);
    }

    #[test]
    fn test_io_error_message() {
        let io_error = io::Error::other("something went wrong");
        let error = FIFError::Io(io_error);

        let msg = "IO Error".red().bold();
        let expected_msg = format!("{msg}: 'something went wrong'");
        assert_eq!(format!("{}", error), expected_msg);
    }
}
