use crate::{FIFError, FIFResult, add_thousands_separator};
use serde::{Deserialize, Serialize};

/// This key will be used by FileInfo and GroupInfo.
///
/// For the FileInfo struct, the hash will be None.
///
/// For the GroupInfo struct, the hash will be Some(blake3).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Key {
    /// Optional hash string (e.g., Blake3, SHA256).
    #[serde(rename = "Hash")]
    pub hash: Option<String>,

    /// The size of the file in bytes.
    #[serde(
        rename = "Size of individual file",
        serialize_with = "add_thousands_separator"
    )]
    pub size: usize,
}

impl Key {
    /// Creates a new `Key`.
    ///
    /// # Errors
    /// Returns `FIFError::ConversionError` if the provided `u64` value
    /// exceeds the maximum value of `usize` on the current architecture.
    pub fn new(value: u64, hash: Option<String>) -> FIFResult<Self> {
        let size = value.try_into().map_err(|_| FIFError::ConversionError {
            from: "u64",
            to: "usize",
        })?;

        Ok(Key { size, hash })
    }

    /// Updates the hash value of the key.
    pub fn set_hash(&mut self, new_hash: Option<String>) {
        self.hash = new_hash;
    }
}

#[cfg(test)]
mod tests_key {
    use super::*;

    /// cargo test -- --show-output test_new_key
    #[test]
    fn test_new_key() -> FIFResult<()> {
        let value: u64 = 12345;
        let valid = Key::new(value, None)?;
        let result = Key {
            size: 12345,
            hash: None,
        };
        println!("key: {valid:#?}");
        assert_eq!(valid, result);
        Ok(())
    }

    /// cargo test -- --show-output test_set_hash
    #[test]
    fn test_set_hash() -> FIFResult<()> {
        let mut key = Key::new(123, None)?;
        println!("key: {key:#?}");

        let string = "foo bar".to_string();
        key.set_hash(Some(string.clone()));
        println!("key: {key:#?}");

        let result = Key {
            size: 123,
            hash: Some(string),
        };
        assert_eq!(key, result);
        Ok(())
    }

    #[test]
    fn test_conversion_error() {
        // On a 32-bit system, this would fail.
        // We can't easily force this test on 64-bit without mocking,
        // but the logic follows try_into() standards.
        let large_value: u64 = u64::MAX;

        // This will only fail on 32-bit platforms
        if usize::BITS < 64 {
            assert!(Key::new(large_value, None).is_err());
        } else {
            assert!(Key::new(large_value, None).is_ok());
        }
    }
}
