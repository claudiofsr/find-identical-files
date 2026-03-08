use criterion::{Criterion, criterion_group, criterion_main};
use find_identical_files::{FIFResult, FileInfo, Key};
use jwalk::{DirEntry, Parallelism, WalkDirGeneric};

/// Updated Type Alias: The client state now holds a Result to capture potential
/// conversion errors (u64 to usize) across parallel threads.
type JwalkResults = Vec<Result<DirEntry<((), Option<FIFResult<FileInfo>>)>, jwalk::Error>>;

// Logic:
// We compare different styles of batch processing in jwalk.
// v1: Functional approach using flatten and for_each.
// v2: Imperative approach using explicit matching.
// v3: Highly chained approach using filter_map and filters.

/// Variant 1: Functional iteration with internal conditional logic.
pub fn process_dir_entries_v1(dir_entry_results: &mut JwalkResults, min_size: u64, max_size: u64) {
    dir_entry_results
        .iter_mut()
        .flatten() // Extract DirEntry from Result
        .filter(|dir_entry| dir_entry.file_type().is_file())
        .for_each(|dir_entry| {
            if let Ok(metadata) = dir_entry.metadata() {
                let file_size = metadata.len();

                if file_size >= min_size && file_size <= max_size {
                    // Map the Key Result to a FileInfo Result
                    let result = Key::new(file_size, None).map(|key| FileInfo {
                        key,
                        path: dir_entry.path(),
                    });
                    dir_entry.client_state = Some(result);
                }
            }
        });
}

/// Variant 2: Imperative style.
/// This style avoids deep iterator chaining which can sometimes be easier to debug.
pub fn process_dir_entries_v2(dir_entry_results: &mut JwalkResults, min_size: u64, max_size: u64) {
    for dir_entry in dir_entry_results.iter_mut().flatten() {
        if dir_entry.file_type().is_file()
            && let Ok(metadata) = dir_entry.metadata()
        {
            let file_size = metadata.len();

            if file_size >= min_size && file_size <= max_size {
                let result = Key::new(file_size, None).map(|key| FileInfo {
                    key,
                    path: dir_entry.path(),
                });
                dir_entry.client_state = Some(result);
            }
        }
    }
}

/// Variant 3: Advanced Iterator chaining.
/// This approach minimizes nested blocks by pre-calculating data in filter_map.
pub fn process_dir_entries_v3(dir_entry_results: &mut JwalkResults, min_size: u64, max_size: u64) {
    dir_entry_results
        .iter_mut()
        .flatten()
        .filter_map(|dir_entry| {
            if dir_entry.file_type().is_file() {
                dir_entry
                    .metadata()
                    .ok()
                    .map(|metadata| (dir_entry, metadata.len()))
            } else {
                None
            }
        })
        .filter(|(_, file_size)| *file_size >= min_size && *file_size <= max_size)
        .for_each(|(dir_entry, file_size)| {
            let result = Key::new(file_size, None).map(|key| FileInfo {
                key,
                path: dir_entry.path(),
            });
            dir_entry.client_state = Some(result);
        });
}

fn benchmark_process_dir_entries(c: &mut Criterion) {
    // Note: "~" is shell-specific. In a cross-platform benchmark,
    // it's better to use a specific test directory or current dir.
    let path = ".";
    let min_size = 0;
    let max_size = 10_000_000;

    let mut group = c.benchmark_group("Jwalk Dir Entries");

    group.warm_up_time(std::time::Duration::from_secs(3));
    group.measurement_time(std::time::Duration::from_secs(10));
    group.sample_size(10);

    // Generic parameter matches the new Result-based client state
    type State = ((), Option<FIFResult<FileInfo>>);

    group.bench_function("process_dir_entries v1", |b| {
        b.iter(|| {
            for _ in WalkDirGeneric::<State>::new(path)
                .parallelism(Parallelism::RayonNewPool(rayon::current_num_threads()))
                .process_read_dir(move |_depth, _path, _state, results| {
                    process_dir_entries_v1(results, min_size, max_size);
                })
            {}
        })
    });

    group.bench_function("process_dir_entries v2", |b| {
        b.iter(|| {
            for _ in WalkDirGeneric::<State>::new(path)
                .parallelism(Parallelism::RayonNewPool(rayon::current_num_threads()))
                .process_read_dir(move |_depth, _path, _state, results| {
                    process_dir_entries_v2(results, min_size, max_size);
                })
            {}
        })
    });

    group.bench_function("process_dir_entries v3", |b| {
        b.iter(|| {
            for _ in WalkDirGeneric::<State>::new(path)
                .parallelism(Parallelism::RayonNewPool(rayon::current_num_threads()))
                .process_read_dir(move |_depth, _path, _state, results| {
                    process_dir_entries_v3(results, min_size, max_size);
                })
            {}
        })
    });

    group.finish();
}

criterion_group!(benches, benchmark_process_dir_entries);
criterion_main!(benches);
