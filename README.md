# find_duplicate_files
Find identical files according to their size and hashing algorithm.

"A hash function is a mathematical algorithm that takes an input (in this case, a file)
and produces a fixed-size string of characters, known as a hash value or checksum.
The hash value acts as a summary representation of the original input.
This hash value is unique (disregarding unlikely [collisions](https://en.wikipedia.org/wiki/Hash_collision))
to the input data, meaning even a slight change in the input will result in a completely different hash value."

Hash algorithm options are:

1. [ahash](https://crates.io/crates/ahash) (used by [hashbrown](https://crates.io/crates/hashbrown))

2. [blake version 3](https://docs.rs/blake3/latest/blake3) (default)

3. [fxhash](https://crates.io/crates/rustc-hash) ([used](https://nnethercote.github.io/2021/12/08/a-brutally-effective-hash-function-in-rust.html) by`FireFox` and `rustc`)

4. [sha256](https://github.com/RustCrypto/hashes)

5. [sha512](https://github.com/RustCrypto/hashes)

find_duplicate_files just reads the files and never changes their contents.
See the function [fn open_file()](https://docs.rs/find_duplicate_files/latest/src/find_duplicate_files/lib.rs.html#62-82) to verify.

## Usage examples

#### 1. To find duplicate files in the current directory, run the command:
```
find_duplicate_files
```

#### 2. Search files in current directory with at least 5 identical files, run the command:
```
find_duplicate_files -n 5
```

With the --min_number (or -n) argument option, set the 'minimum number of identical files'.

With the --max_number (or -N) argument option, set the 'maximum number of identical files'.

If n = 0 or n = 1, all files will be reported.

If n = 2 (default), look for duplicate files or more identical files.

#### 3. To find duplicate files with `fxhash` algorithm and `yaml` format:
```
find_duplicate_files -twa fxhash -r yaml
```

#### 4. To find duplicate files in the `Downloads` directory and redirect the output to a `json` file for further analysis:

```
find_duplicate_files -vi ~/Downloads -r json > fdf.json
```

#### 5. To find duplicate files in the current directory whose size is greater than or equal to 8 bytes:

```
find_duplicate_files -b 8
```

#### 6. To find duplicate files in the current directory whose size is less than or equal to 1024 bytes:

```
find_duplicate_files -B 1024
```

#### 7. To find duplicate files in the current directory whose size is between 8 and 1024 bytes:

```
find_duplicate_files -b 8 -B 1024
```

#### 8. To find duplicate files in the current directory whose size is exactly 1024 bytes:

```
find_duplicate_files -b 1024 -B 1024
```

#### 9. Export duplicate file information from the current directory to an CSV file (fdf.csv).

8.1 The CSV file will be saved in the currenty directory:

```
find_duplicate_files -c .
```
8.2 The CSV file will be saved in the `/tmp` directory:

```
find_duplicate_files --csv_dir=/tmp
```

#### 10. Export duplicate file information from the current directory to an XLSX file (fdf.xlsx).

9.1 The XLSX file will be saved in the `~/Downloads` directory:

```
find_duplicate_files -x ~/Downloads
```
9.2 The XLSX file will be saved in the `/tmp` directory:

```
find_duplicate_files --xlsx_dir=/tmp
```

#### 11. To find duplicate files in the `Downloads` directory and export the result to `/tmp/fdf.xlsx` with the `ahash` algorithm:

```
find_duplicate_files -twi ~/Downloads -x /tmp -a ahash
```

## Help

Type in the terminal `find_duplicate_files -h` to see the help messages and all available options:
```
find identical files according to their size and hashing algorithm

Usage: find_duplicate_files [OPTIONS]

Options:
  -a, --algorithm <ALGORITHM>
          Choose the hash algorithm [default: blake3] [possible values: ahash, blake3, fxhash, sha256, sha512]
  -b, --min_size <MIN_SIZE>
          Set a minimum file size (in bytes) to search for duplicate files
  -B, --max_size <MAX_SIZE>
          Set a maximum file size (in bytes) to search for duplicate files
  -c, --csv_dir <CSV_DIR>
          Set the output directory for the CSV file (fdf.csv)
  -d, --min_depth <MIN_DEPTH>
          Set the minimum depth to search for duplicate files
  -D, --max_depth <MAX_DEPTH>
          Set the maximum depth to search for duplicate files
  -f, --full_path
          Prints full path of duplicate files, otherwise relative path
  -g, --generate <GENERATOR>
          If provided, outputs the completion file for given shell [possible values: bash, elvish, fish, powershell, zsh]
  -i, --input_dir <INPUT_DIR>
          Set the input directory where to search for duplicate files [default: current directory]
  -n, --min_number <MIN_NUMBER>
          Minimum 'number of identical files' to be reported
  -N, --max_number <MAX_NUMBER>
          Maximum 'number of identical files' to be reported
  -o, --omit_hidden
          Omit hidden files (starts with '.'), otherwise search all files
  -r, --result_format <RESULT_FORMAT>
          Print the result in the chosen format [default: personal] [possible values: json, yaml, personal]
  -s, --sort
          Sort result by number of duplicate files, otherwise sort by file size
  -t, --time
          Show total execution time
  -v, --verbose
          Show intermediate runtime messages
  -w, --wipe_terminal
          Wipe (Clear) the terminal screen before listing the duplicate files
  -x, --xlsx_dir <XLSX_DIR>
          Set the output directory for the XLSX file (fdf.xlsx)
  -h, --help
          Print help (see more with '--help')
  -V, --version
          Print version
```

## Building

To build and install from source, run the following command:
```
cargo install find_duplicate_files
```
Another option is to install from github:
```
cargo install --git https://github.com/claudiofsr/find_duplicate_files.git
```

## Mutually exclusive features

#### Walking a directory recursively: jwalk or walkdir.

In general, [jwalk](https://crates.io/crates/jwalk) (default)
is faster than [walkdir](https://crates.io/crates/walkdir).

But if you prefer to use walkdir:
```
cargo install --features walkdir find_duplicate_files
```