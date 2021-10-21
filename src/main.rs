mod filters;
mod input_files;
mod transformer;

use std::collections::HashSet;
use std::path::PathBuf;
use std::str::FromStr;

use crate::filters::{FilterAction, FilterSet};
use crate::input_files::*;
use crate::transformer::{CopyTransformer, InputId, TransformerInstance};

fn main() {
    let jpg_regex = regex::Regex::from_str(".*jpg$").unwrap();
    let flac_glob = glob::Pattern::from_str("*.flac").unwrap();

    let mut filter_set = FilterSet::new();
    filter_set.append_regex(jpg_regex.clone(), FilterAction::Accept);
    filter_set.append_glob(flac_glob.clone(), FilterAction::Accept);

    let input_dir = InputDirectory {
        priority: 100,
        filters: filter_set,
        path: "/home/ociaw/Music".into(),
    };
    let output_dir_path = PathBuf::from("/home/ociaw/Music (processed)");

    std::fs::create_dir_all(&output_dir_path).unwrap();

    let mut copy_filter_set = FilterSet::new();
    copy_filter_set.append_regex(jpg_regex, FilterAction::Accept);

    let mut transformers = Vec::new();
    transformers.push(TransformerInstance { priority: 100, filter: copy_filter_set, transformer: Box::new(CopyTransformer) });

    let mut processed_outputs = HashSet::new();
    let mut unprocessed_files = input_dir.enumerate_files().unwrap();

    println!("Beginning to process {} - {} files", input_dir.path.to_string_lossy(), unprocessed_files.len());
    for transformer in &transformers {
        println!("\tRunning transformer {} - {} files remaining ", "Copy", unprocessed_files.len());
        unprocessed_files.retain(|path| {
            if !transformer.filter.is_acceptable(path) {
                println!("\t\tSkipping {}", path.to_string_lossy());
                return true;
            }

            let transformer = &transformer.transformer;
            let input_id = InputId { dir_path: &input_dir.path, file_path: path };

            let output_id = transformer.determine_output_id(&input_id);
            let output_path = transformer.determine_output_path(&input_id);

            let mut full_input_path = input_dir.path.clone();
            full_input_path.push(path);
            let mut full_output_path = output_dir_path.clone();
            full_output_path.push(output_path);

            match transformer.transform(&full_input_path, &full_output_path) {
                Ok(_) => {
                    processed_outputs.insert(output_id);
                    println!("Transformed {} to {}", full_input_path.to_string_lossy(), full_output_path.to_string_lossy());
                    true
                },
                Err(err) => {
                    eprintln!("Error transforming {} to {}: {}", full_input_path.to_string_lossy(), full_output_path.to_string_lossy(), err.to_string());
                    false
                },
            }
        })
    }
}
