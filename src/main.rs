use find_duplicate_files::*;
use std::time::Instant;

/**
    cargo clippy --features walkdir
    clear && cargo test -- --nocapture
    clear && cargo run -- -h
    clear && cargo run -- -ti ~/Downloads -x /tmp
    cargo run --features walkdir -- -cvts
    cargo doc --open
    cargo b -r && cargo install --path=.
    cargo b -r && cargo install --path=. --features walkdir
*/

// Reading suggestion (not applied here):
// Ordering Requests to Accelerate Disk I/O
// Author: Piotr Kołaczkowski
// https://pkolaczk.github.io/disk-access-ordering/

fn main() -> MyResult<()> {
    set_env_variables();
    let time = Instant::now();
    let arguments = Arguments::build()?;

    // Get useful (duplicate) and useless (non-duplicate) files.
    let all_files: Vec<FileInfo> = get_all_files(&arguments)?;

    if arguments.verbose {
        eprintln!(
            "0. all_files.len(): {}, time_elapsed: {:?}",
            all_files.len(),
            time.elapsed()
        );
    }

    // To skip useless files, 3 procedures will be performed:

    // Procedure 1. Group files by <size> such that the key: (size, None);
    // Ignore filegroups containing only one file.
    let duplicate_size: Vec<GroupInfo> = all_files.get_grouped_files(&arguments, 1);

    if arguments.verbose {
        eprintln!(
            "1. duplicate_size.len(): {}, time_elapsed: {:?}",
            duplicate_size.len(),
            time.elapsed()
        );
    }

    // Procedure 2. Group files by <hash(first_bytes)> such that the key: (size, Some(hash(first_bytes)));
    // Ignore filegroups containing only one file.
    let duplicate_bytes: Vec<GroupInfo> = duplicate_size.get_identical_files(&arguments, 2);

    if arguments.verbose {
        eprintln!(
            "2. duplicate_bytes.len(): {}, time_elapsed: {:?}",
            duplicate_bytes.len(),
            time.elapsed()
        );
    }

    // Procedure 3. Group files by <hash(entire_file)> such that the key: (size, Some(hash(entire_file))).
    // Ignore filegroups containing only one file.
    let mut duplicate_hash: Vec<GroupInfo> = duplicate_bytes.get_identical_files(&arguments, 3);

    if arguments.verbose {
        eprintln!(
            "3. duplicate_hash.len(): {}, time_elapsed: {:?}",
            duplicate_hash.len(),
            time.elapsed()
        );
    }

    // Sort the list of duplicate files.
    duplicate_hash.sort_identical_files(&arguments);

    // Print the duplicated files and the summary information.
    TotalInfo::get_summary(&duplicate_hash, &arguments, all_files.len())
        .print_sumary(&arguments)?;

    // Export duplicate file information simultaneously to CSV and/or XLSX format.
    std::thread::scope(|s| {
        s.spawn(|| -> MyResult<()> {
            if let Some(dir_path) = arguments.csv_dir {
                duplicate_hash.export_to_csv(dir_path)?;
            }
            Ok(())
        });

        s.spawn(|| -> MyResult<()> {
            if let Some(dir_path) = arguments.xlsx_dir {
                duplicate_hash.export_to_xlsx(dir_path)?;
            }
            Ok(())
        });
    });

    if arguments.time {
        println!("Total Execution Time: {:?}", time.elapsed());
    }

    Ok(())
}
