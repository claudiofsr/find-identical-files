use ahash::AHasher;
use blake3::Hasher as Blake3Hasher;
use clap::ValueEnum;
use foldhash::fast::FixedState;
use rustc_hash::FxHasher;
use serde::Serialize;
use sha2::{Digest, Sha256, Sha512};
use std::{
    fmt,
    fs::File,
    hash::{BuildHasher, Hasher},
    io::{BufReader, Read},
    path::PathBuf,
};

use crate::{Arguments, FIFResult, open_file};

/// The default buffer size used for reading files in chunks to calculate hashes.
const BUFFER_SIZE: usize = 64 * 1024; // 64 KB

/// The number of initial bytes to hash when a partial hash is requested.
const FIRST_BYTES: usize = 1024; // 1 KB

/// Hexadecimal characters for converting bytes to hex strings.
const HEX_CHARS: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];

/// Extension trait for byte slices to convert them into hexadecimal strings.
pub trait SliceExtension {
    /// Converts `self` (a byte slice) into its hexadecimal string representation.
    ///
    /// Each byte is represented by two hexadecimal characters.
    ///
    /// # Examples
    /// ```
    /// use find_identical_files::SliceExtension;
    ///
    /// assert_eq!([0x00, 0x0A, 0xFF].as_ref().to_hex_string(), "000aff");
    /// ```
    fn to_hex_string(&self) -> String;
}

impl SliceExtension for [u8] {
    fn to_hex_string(&self) -> String {
        // Allocate space for the string: each byte becomes two hex characters.
        let mut string = String::with_capacity(self.len() * 2);
        for &byte in self {
            // Push the high nibble's hex character
            string.push(HEX_CHARS[((byte >> 4) & 0xF) as usize]);
            // Push the low nibble's hex character
            string.push(HEX_CHARS[(byte & 0xF) as usize]);
        }
        string
    }
}

/// Extension trait for PathBuf to facilitate hashing operations.
pub trait PathBufExtension {
    /// Hashes the content of the file specified by the PathBuf.
    ///
    /// `procedure`:
    /// - `3`: Hash the entire file using the specified algorithm.
    /// - `_`: Hash only the `FIRST_BYTES` of the file using Ahash.
    fn get_hash(&self, arguments: &Arguments, procedure: u8) -> FIFResult<Option<String>>;
}

impl PathBufExtension for PathBuf {
    fn get_hash(&self, arguments: &Arguments, procedure: u8) -> FIFResult<Option<String>> {
        let mut file: File = open_file(self)?;

        let hash_string: String = if procedure == 3 {
            // Hash the entire file
            arguments.algorithm.calculate_hash(file)?
        } else {
            // Hash only the first `FIRST_BYTES` of the file
            calculate_first_bytes_hash(&mut file)?
        };

        Ok(Some(hash_string))
    }
}

/// Calculates a hash based on the first `FIRST_BYTES` of the file.
///
/// This uses `AHasher` for speed on a small, fixed-size chunk of data.
///
/// ### Arguments
/// * `file` - A mutable reference to the `File` to be hashed.
///
/// ### Returns
/// A `FIFResult` containing the hash as a `String` if successful,
/// or an `Err(MyError)` if an I/O error occurs.
fn calculate_first_bytes_hash(file: &mut File) -> FIFResult<String> {
    let mut buffer = [0_u8; FIRST_BYTES];

    // Read up to FIRST_BYTES bytes. `read` returns the number of bytes read.
    let count = file.read(&mut buffer)?;

    let mut hasher = AHasher::default();
    hasher.write(&buffer[..count]); // Hash only the bytes that were actually read.
    Ok(hasher.finish().to_string())
}

/// Enum representing supported hash algorithms.
///
/// Implements `Display` to allow serializing the enum variant names in PascalCase,
/// and `ValueEnum` for use with `clap`.
#[derive(Debug, Default, Clone, Copy, ValueEnum, Serialize)]
#[serde(rename_all = "PascalCase")] // Serialize enum variants to PascalCase strings
pub enum Algorithm {
    Ahash,
    #[default]
    Blake3, // Blake3 is generally a good default for speed and security
    Foldhash,
    Fxhash,
    SHA256, // Cryptographic hash
    SHA512, // Cryptographic hash
}

/// Implements `fmt::Display` for `Algorithm` to display variant names in PascalCase.
impl fmt::Display for Algorithm {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        // Leverage serde's `serialize` method to get the PascalCase string representation.
        self.serialize(formatter).map_err(|_| fmt::Error) // Convert serde::Error to fmt::Error
    }
}

impl Algorithm {
    /// Calculate file hash using some algorithm
    pub fn calculate_hash(&self, file: File) -> FIFResult<String> {
        let reader: BufReader<File> = BufReader::with_capacity(BUFFER_SIZE, file);

        match self {
            Algorithm::Ahash => get_ahash(reader),
            Algorithm::Blake3 => get_blake3(reader),
            Algorithm::Fxhash => get_fxhash(reader),
            Algorithm::Foldhash => get_foldhash(reader),
            Algorithm::SHA256 => get_sha256(reader),
            Algorithm::SHA512 => get_sha512(reader),
        }
    }
}

/// Calculates the aHash from Path.
///
/// <https://crates.io/crates/ahash>
fn get_ahash(mut reader: impl Read) -> FIFResult<String> {
    let mut buffer = [0_u8; BUFFER_SIZE];
    let mut hasher = AHasher::default();

    loop {
        // read up to BUFFER_SIZE bytes to buffer
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.write(&buffer[..count]);
    }

    Ok(hasher.finish().to_string())
}

/// Calculates the Blake3 hash from Path.
///
/// <https://docs.rs/blake3/latest/blake3>
fn get_blake3<R>(mut reader: R) -> FIFResult<String>
where
    R: Read,
{
    let mut buffer = [0_u8; BUFFER_SIZE];
    let mut hasher = Blake3Hasher::new();

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    Ok(hasher.finalize().to_string())
}

/// Calculates the FoldHash from Path.
///
/// A fast, non-cryptographic, minimally DoS-resistant hashing algorithm for Rust.
///
/// <https://crates.io/crates/foldhash>
fn get_foldhash<R>(mut reader: R) -> FIFResult<String>
where
    R: Read,
{
    let mut buffer = [0_u8; BUFFER_SIZE];
    let mut hasher = FixedState::default().build_hasher();

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.write(&buffer[..count]);
    }

    Ok(hasher.finish().to_string())
}

/// Calculates the FxHash from Path.
///
/// Fast, non-cryptographic hash function used by rustc and Firefox.
///
/// (Indeed, the Fx is short for “Firefox”.)
///
/// <https://crates.io/crates/rustc-hash>
///
/// <https://nnethercote.github.io/2021/12/08/a-brutally-effective-hash-function-in-rust.html>
fn get_fxhash<R>(mut reader: R) -> FIFResult<String>
where
    R: Read,
{
    let mut buffer = [0_u8; BUFFER_SIZE];
    let mut hasher = FxHasher::default();

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.write(&buffer[..count]);
    }

    Ok(hasher.finish().to_string())
}

/// Calculates the SHA2 256 from Path.
///
/// Verify with
///
/// openssl dgst -sha256 Some_File
///
/// <https://github.com/RustCrypto/hashes/tree/master/sha2>
fn get_sha256<R>(mut reader: R) -> FIFResult<String>
where
    R: Read,
{
    let mut buffer = [0_u8; BUFFER_SIZE];
    let mut hasher = Sha256::new();

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        // `update` can be called repeatedly and is generic over `AsRef<[u8]>`
        hasher.update(&buffer[..count]);
    }

    // Note that calling `finalize()` consumes hasher
    Ok(hasher.finalize().to_hex_string())
}

/// Calculates the SHA2 512 from Path.
///
/// Verify with
///
/// openssl dgst -sha512 Some_File
///
/// <https://github.com/RustCrypto/hashes/tree/master/sha2>
fn get_sha512<R>(mut reader: R) -> FIFResult<String>
where
    R: Read,
{
    let mut buffer = [0_u8; BUFFER_SIZE];
    let mut hasher = Sha512::new();

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        // `update` can be called repeatedly and is generic over `AsRef<[u8]>`
        hasher.update(&buffer[..count]);
    }

    // Note that calling `finalize()` consumes hasher
    Ok(hasher.finalize().to_hex_string())
}

#[cfg(test)]
mod tests_algo {
    use super::*;
    use std::{
        cmp,
        io::{BufReader, Cursor, Write},
    };
    use tempfile::NamedTempFile; // Temporary file creation for tests

    #[test]
    fn test_slice_to_hex_string_empty() {
        assert_eq!([].as_ref().to_hex_string(), "");
    }

    #[test]
    fn test_slice_to_hex_string_simple() {
        assert_eq!([0x00].as_ref().to_hex_string(), "00");
        assert_eq!([0x0F].as_ref().to_hex_string(), "0f");
        assert_eq!([0x10].as_ref().to_hex_string(), "10");
        assert_eq!([0xFF].as_ref().to_hex_string(), "ff");
    }

    #[test]
    fn test_slice_to_hex_string_multiple_bytes() {
        assert_eq!(
            [0xDE, 0xAD, 0xBE, 0xEF].as_ref().to_hex_string(),
            "deadbeef"
        );
        assert_eq!([10, 20, 30, 40, 255].as_ref().to_hex_string(), "0a141e28ff");
    }

    /// Helper to create a temporary file with content
    fn create_temp_file(content: &[u8]) -> FIFResult<NamedTempFile> {
        let mut file = NamedTempFile::new()?;
        file.write_all(content)?;
        file.flush()?; // Ensure all data is written to the disk
        Ok(file)
    }

    // --- Hashing Tests ---

    /// A helper function to hash a byte slice using a given algorithm.
    /// This avoids creating temp files for small strings.
    fn hash_bytes_with_algorithm(content: &[u8], algorithm: Algorithm) -> FIFResult<String> {
        let cursor = Cursor::new(content.to_vec());
        let reader = BufReader::new(cursor); // Use BufReader for consistency with file hashing
        match algorithm {
            Algorithm::Ahash => get_ahash(reader),
            Algorithm::Blake3 => get_blake3(reader),
            Algorithm::Fxhash => get_fxhash(reader),
            Algorithm::Foldhash => get_foldhash(reader),
            Algorithm::SHA256 => get_sha256(reader),
            Algorithm::SHA512 => get_sha512(reader),
        }
    }

    #[test]
    fn test_get_ahash() -> FIFResult<()> {
        // Known ahash value for "hello world" will vary between systems/versions,
        // as AHash is non-cryptographic and relies on specific CPU instructions.
        // So, we'll hash an empty string and a known string and assert they are different.
        let empty_hash = hash_bytes_with_algorithm(b"", Algorithm::Ahash)?;
        let hello_hash = hash_bytes_with_algorithm(b"hello world", Algorithm::Ahash)?;
        assert!(!empty_hash.is_empty());
        assert!(!hello_hash.is_empty());
        assert_ne!(empty_hash, hello_hash);
        Ok(())
    }

    #[test]
    /// cargo test -- --show-output blake3
    fn test_get_blake3() -> FIFResult<()> {
        let empty_hash = hash_bytes_with_algorithm(b"", Algorithm::Blake3)?;
        let hello_hash = hash_bytes_with_algorithm(b"hello world", Algorithm::Blake3)?;

        println!("blake3 empty_hash: {empty_hash}");
        println!("blake3 hello_hash: {hello_hash}");

        // Blake3 has official reference values
        assert_eq!(
            empty_hash,
            "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"
        );
        assert_eq!(
            hello_hash,
            "d74981efa70a0c880b8d8c1985d075dbcbf679b99a5f9914e5aaf96b831a9e24"
        );
        // Command: `echo -n "hello world" | b3sum`
        // Result: d74981efa70a0c880b8d8c1985d075dbcbf679b99a5f9914e5aaf96b831a9e24
        Ok(())
    }

    #[test]
    fn test_get_sha256() -> FIFResult<()> {
        let empty_hash = hash_bytes_with_algorithm(b"", Algorithm::SHA256)?;
        let hello_hash = hash_bytes_with_algorithm(b"hello world", Algorithm::SHA256)?;

        // SHA256 has official reference values
        assert_eq!(
            empty_hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            hello_hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
        // Command: `echo -n "hello world" | sha256sum`
        // Command: `echo -n "hello world" | shasum -a 256`
        // Command: `echo -n "hello world" | openssl dgst -sha256`
        // Result: 960a483cea18c1049a0b878ad1032de3a40993500652a43fbdb7450c40b46993
        Ok(())
    }

    #[test]
    fn test_get_sha512() -> FIFResult<()> {
        let empty_hash = hash_bytes_with_algorithm(b"", Algorithm::SHA512)?;
        let hello_hash = hash_bytes_with_algorithm(b"hello world", Algorithm::SHA512)?;

        // SHA512 has official reference values
        assert_eq!(
            empty_hash,
            "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e"
        );
        assert_eq!(
            hello_hash,
            "309ecc489c12d6eb4cc40f50c902f2b4d0ed77ee511a7c7a9bcd3ca86d4cd86f989dd35bc5ff499670da34255b45b0cfd830e81f605dcf7dc5542e93ae9cd76f"
        );
        // Command: `echo -n "hello world" | sha512sum`
        // Command: `echo -n "hello world" | shasum -a 512`
        // Command: `echo -n "hello world" | openssl dgst -sha512`
        // Result: 960a483cea18c1049a0b878ad1032de3a40993500652a43fbdb7450c40b46993
        Ok(())
    }

    #[test]
    fn test_pathbuf_extension_first_bytes_hash() -> FIFResult<()> {
        let content = b"This is a longer test string that should be truncated for the first bytes hash calculation.";
        let temp_file = create_temp_file(content)?;
        let path = temp_file.path().to_path_buf();

        let mut args = Arguments::build()?;
        args.algorithm = Algorithm::Ahash;

        // Assuming procedure 0-2 trigger first_bytes_hash
        let hash = path.get_hash(&args, 0)?;

        // Re-calculate the expected hash manually using only the first FIRST_BYTES.
        let mut expected_hasher = AHasher::default();
        let truncated_content = &content[..cmp::min(content.len(), FIRST_BYTES)];
        expected_hasher.write(truncated_content);
        let expected_hash = expected_hasher.finish().to_string();

        assert_eq!(hash, Some(expected_hash));
        Ok(())
    }

    #[test]
    fn test_pathbuf_extension_full_file_hash() -> FIFResult<()> {
        let content =
            b"This is a longer test string that should trigger the full file hash calculation.";
        let temp_file = create_temp_file(content)?;
        let path = temp_file.path().to_path_buf();

        let mut args = Arguments::build()?;
        args.algorithm = Algorithm::Blake3;

        // Procedure 3 triggers full file hash
        let hash = path.get_hash(&args, 3)?.unwrap();

        // Blake3 has official reference values
        let expected_hash = "960a483cea18c1049a0b878ad1032de3a40993500652a43fbdb7450c40b46993";
        // Command: `echo -n "This is a longer test string that should trigger the full file hash calculation." | b3sum`
        // Result: 960a483cea18c1049a0b878ad1032de3a40993500652a43fbdb7450c40b46993
        assert_eq!(hash, expected_hash);
        Ok(())
    }

    #[test]
    fn test_pathbuf_extension_empty_file_full_hash() -> FIFResult<()> {
        let temp_file = create_temp_file(b"")?;
        let path = temp_file.path().to_path_buf();

        let mut args = Arguments::build()?;
        args.algorithm = Algorithm::SHA256;

        let hash = path.get_hash(&args, 3)?.unwrap();
        // SHA256 of empty string
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        // Command: `echo -n "" | sha256sum`
        // Result: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        Ok(())
    }
}
