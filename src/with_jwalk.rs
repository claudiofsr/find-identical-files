use crate::{Arguments, FIFResult, FileInfo, Key, get_path};
use jwalk::{DirEntry, Parallelism, WalkDirGeneric};
use std::path::PathBuf;

/// Collects all files within the specified directory into a single Vector.
///
/// This function uses `jwalk` for high-performance, multi-threaded directory traversal.
/// It filters files based on the criteria provided in `Arguments` (size, depth, hidden status).
pub fn get_all_files(arguments: &Arguments) -> FIFResult<Vec<FileInfo>> {
    let path: PathBuf = get_path(arguments)?;

    let min_size: u64 = arguments.min_size;
    let max_size: u64 = arguments.max_size;

    // The client state now holds an Option of a Result.
    // This allows us to bubble up errors that happen inside the parallel threads.
    let jwalk = WalkDirGeneric::<((), Option<FIFResult<FileInfo>>)>::new(path)
        .skip_hidden(arguments.omit_hidden)
        .min_depth(arguments.min_depth)
        .max_depth(arguments.max_depth)
        .parallelism(Parallelism::RayonNewPool(rayon::current_num_threads()))
        .process_read_dir(move |_depth, _path, _read_dir_state, dir_entry_results| {
            process_dir_entries(dir_entry_results, min_size, max_size);
        });

    // Used filter_map(|result| result.ok()) to cleanly discard walking errors
    // (like temporary files disappearing mid-walk) without breaking the entire collection process.

    // We collect into a Result<Vec<FileInfo>, FIFError>.
    // If any entry contains an Err, collect will stop and return that error immediately.
    let all_files: Vec<FileInfo> = jwalk
        .into_iter()
        .filter_map(|result| result.ok()) // Ignore jwalk walking errors
        .flat_map(|dir_entry| dir_entry.client_state) // Get the Result<FileInfo, FIFError>
        .collect::<FIFResult<Vec<FileInfo>>>()?; // Propagate the first error found

    Ok(all_files)
}

/// Type alias for jwalk results to improve readability.
type JwalkResults = Vec<Result<DirEntry<((), Option<FIFResult<FileInfo>>)>, jwalk::Error>>;

/// Processes directory entries and populates the client state with either a FileInfo or a FIFError.
fn process_dir_entries(dir_entry_results: &mut JwalkResults, min_size: u64, max_size: u64) {
    dir_entry_results
        .iter_mut()
        .flatten() // Skip jwalk-specific errors
        .filter(|dir_entry| dir_entry.file_type().is_file())
        .for_each(|dir_entry| {
            if let Ok(metadata) = dir_entry.metadata() {
                let file_size = metadata.len();

                if file_size >= min_size && file_size <= max_size {
                    // We attempt to create a Key. If it fails, we store the Err in client_state.
                    let result = Key::new(file_size, None).map(|key| FileInfo {
                        key,
                        path: dir_entry.path(),
                    });

                    dir_entry.client_state = Some(result);
                }
            }
        });
}
