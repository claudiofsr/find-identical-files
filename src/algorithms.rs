use crate::{
    Arguments,
    MyResult,
    args::Algorithm::*,
};
use ahash::AHasher;
use blake3::Hasher as Blake3Hasher;
use std::{
    fs::{self, File},
    hash::Hasher,
    path::{Path, PathBuf},
    io::{Read, BufReader},
};
use ring::digest::{
    self,
    Context,
    SHA256 as DIGEST_SHA256,
    SHA512 as DIGEST_SHA512,
};
use rustc_hash::FxHasher;

const FIRST_BYTES: usize = 64;
const BUFFER_SIZE: usize = 64 * 1024;
const STACK_SIZE:  usize = 64 * 1024 * 1024;

const HEX: [char; 16] = [
'0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
'a', 'b', 'c', 'd', 'e', 'f',
];

/**
If `thread '<unknown>' has overflowed its stack`, set the stack size to a new value.

The default rustc stack size for main thread is currently `8 * 1024 * 1024`.

Stack size can be changed during compile time.

<https://github.com/rust-lang/rust/blob/master/compiler/rustc_interface/src/util.rs#L132>

<https://github.com/rust-lang/rust/issues/111897>

<https://doc.rust-lang.org/stable/std/thread/index.html#stack-size>
*/
pub fn set_env_variables() {
    std::env::set_var("RUST_MIN_STACK", STACK_SIZE.to_string());
}

pub trait SliceExtension {
    fn to_hex_string(self) -> String;
}

impl SliceExtension for &[u8] {
    fn to_hex_string(self) -> String {
        self.iter()
            .flat_map(|byte| {
                let a: char = HEX[(*byte as usize)/16];
                let b: char = HEX[(*byte as usize)%16];
                [a, b]
            })
            .collect()
    }
}

pub trait PathBufExtension {
    fn get_hash(&self, opt_arguments: Option<&Arguments>) -> MyResult<Option<String>>;
}

impl PathBufExtension for PathBuf {
    /// Hash the first few bytes or the entire file.
    ///
    /// <https://rust-lang-nursery.github.io/rust-cookbook/cryptography/hashing.html>
    fn get_hash(&self, opt_arguments:Option<&Arguments>) -> MyResult<Option<String>>
    {
        let mut file: File = open_file(self)?;

        let hash: String = match opt_arguments {
            Some(arguments) => {
                // Hash the entire file with your chosen hashing algorithm.
                let reader: BufReader<File> = BufReader::with_capacity(BUFFER_SIZE, file);
                match arguments.algorithm {
                    Ahash  => get_ahash(reader)?,
                    Blake3 => get_blake3(reader)?,
                    Fxhash => get_fxhash(reader)?,
                    SHA256 => get_sha(reader, &DIGEST_SHA256)?,
                    SHA512 => get_sha(reader, &DIGEST_SHA512)?,
                }
            },
            None => {
                // Get only the first few bytes to hash.
                let mut buffer = [0_u8; FIRST_BYTES];
                let count = file.read(&mut buffer)?;

                let mut hasher = AHasher::default();
                hasher.write(&buffer[..count]);
                hasher.finish().to_string()

                //buffer[..count].to_hex_string()
            }
        };

        Ok(Some(hash))
    }
}

/// File is an object providing access to an open file on the filesystem.
pub fn open_file<P>(path: P) -> MyResult<File>
where
    P: AsRef<Path> + std::marker::Copy + std::fmt::Debug,
{
    let file: File = match fs::OpenOptions::new()
        .read(true)
        .write(false) // This option, when false, will indicate that the file should not be writable if opened.
        .create(false)
        .open(path)
        {
            Ok(file) => file,
            Err(error) => {
                eprintln!("Failed to open file {path:?}");
                eprintln!("Perhaps some temporary files no longer exist!");
                eprintln!("Or lack of permission to read this file!");
                panic!("{error}");
            }
        };

    Ok(file)
}

/// Calculates the aHash from Path.
///
/// <https://crates.io/crates/ahash>
fn get_ahash<R: Read>(mut reader: R) -> MyResult<String> {
    let mut buffer = [0_u8; BUFFER_SIZE];
    let mut hasher = AHasher::default();

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.write(&buffer[..count]);
    }

    let hash: String = hasher.finish().to_string();

    Ok(hash)
}

/// Calculates the Blake3 hash from Path.
///
/// <https://docs.rs/blake3/latest/blake3>
fn get_blake3<R>(mut reader: R) -> MyResult<String>
where
    R: Read
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

    let hash: String = hasher.finalize().to_string();

    Ok(hash)
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
fn get_fxhash<R: Read>(mut reader: R) -> MyResult<String> {
    let mut buffer = [0_u8; BUFFER_SIZE];
    let mut hasher = FxHasher::default();

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.write(&buffer[..count]);
    }

    let hash: String = hasher.finish().to_string();

    Ok(hash)
}

/// Calculates the sha256 or sha512 hash from Path.
///
/// <https://docs.rs/ring/latest/ring/digest/fn.digest.html>
///
/// <https://rust-lang-nursery.github.io/rust-cookbook/cryptography/hashing.html>
fn get_sha<R: Read>(mut reader: R, algorithm: &'static digest::Algorithm) -> MyResult<String> {
    let mut buffer = [0_u8; BUFFER_SIZE];
    let mut hasher = Context::new(algorithm);

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    let digest = hasher.finish();

    // Vec<u8> in hex representation:
    let hash: String = digest.as_ref().to_hex_string();

    Ok(hash)
}
