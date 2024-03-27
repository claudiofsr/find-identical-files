use serde::Serialize;
use crate::{
    add_thousands_separator,
    to_usize,
};

/// This key will be used by FileInfo and GroupInfo.
///
/// For the FileInfo struct, the hash will be None.
///
/// For the GroupInfo struct, the hash will be Some(blake3).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct Key {
    /// Individual file size (in bytes)
    #[serde(serialize_with = "add_thousands_separator")]
    pub size: usize,
    /// Blake3 hash
    pub hash: Option<String>,
}

impl Key {
    pub fn new<T>(value: T, hash: Option<String>) -> Self
    where 
        T: TryInto<usize>,
    {
        Key {
            size: to_usize(value),
            hash,
        }
    }

    /// Set hash value
    pub fn set_hash(&mut self, hash: Option<String>) {
        self.hash = hash;
    }
}

#[cfg(test)]
mod test_key {
    use super::*;

    /// cargo test -- --show-output test_new_key
    #[test]
    fn test_new_key() {
        let value: u64 = 12345;
        let valid = Key::new(value, None);
        let result = Key {
            size: 12345,
            hash: None,
        };
        println!("key: {valid:#?}");
        assert_eq!(valid, result);
    }
}
