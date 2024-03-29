use find_duplicate_files::*;
use std::time::Instant;

/**
    cargo clippy --features walkdir
    clear && cargo test -- --nocapture
    clear && cargo run -- -h
    cargo run --features walkdir -- -cvts
    cargo doc --open
    cargo b -r && cargo install --path=.
    cargo b -r && cargo install --features walkdir --path=.
*/

// Reading suggestion (not applied here):
// Ordering Requests to Accelerate Disk I/O
// Autor: Piotr Kołaczkowski
// https://pkolaczk.github.io/disk-access-ordering/

fn main() -> MyResult<()> {

    set_env_variables();
    let time = Instant::now();
    let arguments = Arguments::build();

    // Get useful (duplicate) and useless (non-duplicate) files.
    let all_files: Vec<FileInfo> = get_all_files(&arguments)?;

    if arguments.verbose {
        eprintln!("0. all_files.len(): {}, time_elapsed: {:?}", all_files.len(), time.elapsed());
    }

    // To skip useless files, 3 procedures will be performed:

    // 1. Group files by <size> such that the key: (size, None);
    // Ignore filegroups containing only one file.
    let duplicate_size: Vec<GroupInfo> = get_grouped_files(&all_files);

    if arguments.verbose {
        eprintln!("1. duplicate_size.len(): {}, time_elapsed: {:?}", duplicate_size.len(), time.elapsed());
    }

    // 2. Group files by <first bytes> such that the key: (size, Some(bytes));
    // Ignore filegroups containing only one file.
    let duplicate_bytes: Vec<GroupInfo> = get_duplicate_files(&duplicate_size, None);

    if arguments.verbose {
        eprintln!("2. duplicate_bytes.len(): {}, time_elapsed: {:?}", duplicate_bytes.len(), time.elapsed());
    }

    // 3. Group files by <hash> such that the key: (size, Some(hash)).
    // Ignore filegroups containing only one file.
    let mut duplicate_hash: Vec<GroupInfo> = get_duplicate_files(&duplicate_bytes, Some(&arguments));

    if arguments.verbose {
        eprintln!("3. duplicate_hash.len(): {}, time_elapsed: {:?}", duplicate_hash.len(), time.elapsed());
    }

    // Sort the list of duplicate files.
    duplicate_hash.sort_duplicate_files(&arguments);

    // Print the duplicated files and the summary information.
    TotalInfo::get_summary(&duplicate_hash, &arguments, all_files.len())
        .print_sumary(&arguments)?;

    if arguments.time {
        println!("Total Execution Time: {:?}", time.elapsed());
    }

    Ok(())
}