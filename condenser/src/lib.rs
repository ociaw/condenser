mod filters;
mod input_files;
mod transformer;

use std::collections::HashSet;
use std::fs::read_dir;
use std::path::Path;
use std::path::PathBuf;

pub use crate::filters::*;
pub use crate::input_files::*;
pub use crate::transformer::*;

// TODO: Create an object to hold this data.

/// Runs transformations on the provided input directories using the provided
/// transformers, outputing to the directory specified by output_dir_path.
pub fn run_transformations<'a, DirIter, P>(
    transformers: &mut Vec<TransformerInstance>,
    input_dirs: DirIter,
    output_dir_path: P,
) where
    DirIter: IntoIterator<Item = &'a InputDirectory>,
    P: AsRef<Path>,
{
    // Ensure that transformers are sorted by priority, highest first.
    transformers.sort_by(|t1, t2| t1.priority.cmp(&t2.priority).reverse());
    let output_dir_path = output_dir_path.as_ref();
    let mut claimed_outputs = HashSet::new();
    let mut output_paths = HashSet::new();

    // Enumerate all input directories and enqueue each matching file with its transformer.
    // TODO: Might be better to have the queue separate from the TransformerInstance object.
    for input_dir in input_dirs {
        let input_path = &input_dir.path;
        let mut unprocessed_files = input_dir.enumerate_files().unwrap();

        println!(
            "Beginning to claim {1} files in {0}",
            input_path.to_string_lossy(),
            unprocessed_files.len()
        );
        for transformer in transformers.iter_mut() {
            let count =
                transformer.claim_outputs(input_path, &mut unprocessed_files, &mut claimed_outputs, &mut output_paths);
            println!(
                "  Transformer '{}' claimed {} files - {} remaining ",
                &transformer.name,
                count,
                unprocessed_files.len()
            );
        }
    }

    // Ensure the output directory exists
    std::fs::create_dir_all(output_dir_path).unwrap_or_else(|_| {
        panic!(
            "Failed to create output directory: {}",
            output_dir_path.to_string_lossy()
        )
    });

    println!("Deleting orphaned files...");
    // Delete any orphans from the output directory
    if let Err(err) = delete_orphans(output_dir_path, output_dir_path, &output_paths) {
        println!("Failed to delete orphaned files: '{}'. Transformations will continue.", err)
    }

    // Run the tranformers - this can potentially be done in parallel for each transformer,
    // since they should be independent from each other.
    println!("Running {} transformer(s)...", transformers.len());
    for transformer in transformers {
        let errors = transformer.process_queues(output_dir_path);
        println!(
            "  Transformer '{}' processing completed - {} error(s)",
            &transformer.name,
            errors.len()
        );
        for error in errors {
            println!("    `{}' - {}", error.0.to_string_lossy(), error.1)
        }
    }
}

fn delete_orphans(root_dir: &Path, current_dir: &Path, allowed_files: &HashSet<PathBuf>) -> Result<(), std::io::Error> {
    // TODO: This needs a lot more configuration options.
    for entry in read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            delete_orphans(root_dir, &path, allowed_files)?;
            continue;
        } else if !path.is_file() {
            // Skip things that aren't files and aren't paths
            continue;
        }

        if let Ok(relative_path) = path.strip_prefix(root_dir) {
            // Skip any files not matching the root prefix
            if !allowed_files.contains(relative_path) {
                println!("Deleting {}", relative_path.to_string_lossy());
                std::fs::remove_file(path)?;
            }
        }
    }
    Ok(())
}
