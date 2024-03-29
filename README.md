# find_duplicate_files
Find duplicate files according to their size and hashing algorithm.

"A hash function is a mathematical algorithm that takes an input (in this case, a file)
and produces a fixed-size string of characters, known as a hash value or checksum.
The hash value acts as a summary representation of the original input.
This hash value is unique (disregarding unlikely [collisions](https://en.wikipedia.org/wiki/Hash_collision))
to the input data, meaning even a slight change in the input will result in a completely different hash value."

Hash algorithm options are:

1. [ahash](https://crates.io/crates/ahash) (used by [hashbrown](https://crates.io/crates/hashbrown))

2. [blake version 3](https://docs.rs/blake3/latest/blake3) (default)

3. [fxhash](https://crates.io/crates/rustc-hash) ([used](https://nnethercote.github.io/2021/12/08/a-brutally-effective-hash-function-in-rust.html) by`FireFox` and `rustc`)

4. [sha256](https://crates.io/crates/ring)

5. [sha512](https://crates.io/crates/ring)

find_duplicate_files just reads the files and never changes their contents.
See the function [fn open_file()](https://docs.rs/find_duplicate_files/latest/src/find_duplicate_files/algorithms.rs.html#106-126) to verify.

## Usage examples

1. To find duplicate files in the current directory, run the command:
```
find_duplicate_files
```

2. To find duplicate files with `fxhash` algorithm and `yaml` format:
```
find_duplicate_files -csta fxhash -r yaml
```

3. To find duplicate files in the `Downloads` directory and redirect the output to a `json` file for further analysis:

```
find_duplicate_files -p ~/Downloads -r json > fdf.json
```

## Help

Type in the terminal `find_duplicate_files -h` to see the help messages and all available options:
```
find duplicate files according to their size and hashing algorithm

Usage: find_duplicate_files [OPTIONS]

Options:
  -a, --algorithm <ALGORITHM>
          Choose the hash algorithm [default: blake3] [possible values: ahash, blake3, fxhash, sha256, sha512]
  -c, --clear_terminal
          Clear the terminal screen before listing the duplicate files
  -f, --full_path
          Prints full path of duplicate files, otherwise relative path
  -g, --generate <GENERATOR>
          If provided, outputs the completion file for given shell [possible values: bash, elvish, fish, powershell, zsh]
  -m, --max_depth <MAX_DEPTH>
          Set the maximum depth to search for duplicate files
  -k, --min_size <MIN_SIZE>
          Set a minimum file size to search for duplicate files [default: 0]
  -o, --omit_hidden
          Omit hidden files (starts with '.'), otherwise search all files
  -p, --path <PATH>
          Set the path where to look for duplicate files, otherwise use the current directory
  -r, --result_format <RESULT_FORMAT>
          Print the result in the chosen format [default: personal] [possible values: json, yaml, personal]
  -s, --sort
          Sort result by file size, otherwise sort by number of duplicate files
  -t, --time
          Show total execution time
  -v, --verbose
          Show intermediate runtime messages
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