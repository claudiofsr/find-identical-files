use find_identical_files::*;
use std::time::Instant;
//use futures::executor::block_on;

/**
    cargo clippy --features walkdir
    clear && cargo test -- --nocapture
    clear && cargo run -- -h
    clear && cargo run -- -tvi ~/Downloads -x /tmp -r json > /tmp/fif.json
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

    // Get useful (identical) and useless (non-identical) files.
    let all_files: Vec<FileInfo> = get_all_files(&arguments)?;

    if arguments.verbose {
        eprintln!(
            "0. {:<43}: {:>10}, time_elapsed: {:?}",
            "Total number of files",
            all_files.len(),
            time.elapsed()
        );
    }

    // To skip useless files, 3 procedures will be performed:

    // Procedure 1. Group files by <size> such that the key: (size, None);
    // Ignore filegroups containing only one file.
    let identical_size: Vec<GroupInfo> = all_files.get_grouped_files(&arguments, 1);

    if arguments.verbose {
        eprintln!(
            "1. {:<43}: {:>10}, time_elapsed: {:?}",
            "Number of files of identical size",
            identical_size.len(),
            time.elapsed()
        );
    }

    // Procedure 2. Group files by <hash(first_bytes)> such that the key: (size, Some(hash(first_bytes)));
    // Ignore filegroups containing only one file.
    let identical_bytes: Vec<GroupInfo> = identical_size.get_identical_files(&arguments, 2);

    if arguments.verbose {
        eprintln!(
            "2. {:<43}: {:>10}, time_elapsed: {:?}",
            "Number of files with identical first bytes",
            identical_bytes.len(),
            time.elapsed()
        );
    }

    /*
    // For testing purposes only:
    // https://rust-lang.github.io/async-book/01_getting_started/04_async_await_primer.html
    let groups = block_on(all(&identical_bytes, &arguments));
    let mut identical_hash: Vec<GroupInfo> = [
        groups.0,
        groups.1,
        groups.2,
        groups.3,
    ].concat();
    */

    // Procedure 3. Group files by <hash(entire_file)> such that the key: (size, Some(hash(entire_file))).
    // Ignore filegroups containing only one file.
    let mut identical_hash: Vec<GroupInfo> = identical_bytes.get_identical_files(&arguments, 3);

    if arguments.verbose {
        eprintln!(
            "3. {:<43}: {:>10}, time_elapsed: {:?}",
            "Number of files with identical hashes",
            identical_hash.len(),
            time.elapsed()
        );
    }

    // Sort the list of identical files.
    identical_hash.sort_identical_files(&arguments);

    // Print the identical files and the summary information.
    TotalInfo::get_summary(&identical_hash, &arguments, all_files.len())
        .print_sumary(&arguments)?;

    // Export identical file information simultaneously to CSV and/or XLSX format.
    std::thread::scope(|s| {
        s.spawn(|| -> MyResult<()> {
            if let Some(dir_path) = arguments.csv_dir {
                identical_hash.export_to_csv(dir_path)?;
            }
            Ok(())
        });

        s.spawn(|| -> MyResult<()> {
            if let Some(dir_path) = arguments.xlsx_dir {
                identical_hash.export_to_xlsx(dir_path)?;
            }
            Ok(())
        });
    });

    if arguments.time {
        println!("Total Execution Time: {:?}", time.elapsed());
    }

    Ok(())
}

/*
async fn analise(d: &[GroupInfo], arguments: &Arguments) -> Vec<GroupInfo> {
    // Procedure 3. Group files by <hash(entire_file)> such that the key: (size, Some(hash(entire_file))).
    // Ignore filegroups containing only one file.
    let identical_hash: Vec<GroupInfo> = d.get_identical_files(arguments, 3);

    identical_hash
}

async fn all(d: &[GroupInfo], arguments: &Arguments) -> (Vec<GroupInfo>, Vec<GroupInfo>, Vec<GroupInfo>, Vec<GroupInfo>) {
    let group_number = d.len();
    let g: Vec<&[GroupInfo]> = d.chunks(group_number / 4).collect();
    futures::join!(
        analise(g[0], arguments),
        analise(g[1], arguments),
        analise(g[2], arguments),
        analise(g[3], arguments),
    )
    //(side_a, side_b)
}
*/
